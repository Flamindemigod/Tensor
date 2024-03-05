pub mod channel;
pub mod client;
pub mod http;
pub mod message;
pub mod server;
pub mod websocket;

use anyhow::Result;
use channel::{interaction_channel, ClientInteractions, Clients, ServerInteractions};
use std::{path::PathBuf, process::exit};

use argh::FromArgs;
use http::http_main;
use server::Server;
use websocket::websocket_main;

#[derive(FromArgs, Debug)]
///Tensor Server
struct Args {
    /// generate new client with name
    #[argh(option, short = 'n', long = "new", arg_name = "name")]
    name: Option<String>,

    /// path to config directory. Defaults to "."
    #[argh(positional, default = "PathBuf::from(\".\")", greedy)]
    dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let _ = ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        exit(0)
    });

    //Init Server:
    let mut server = Server::init_server(args.dir);
    if let Some(username) = args.name {
        server.new_client(&username);
        return Ok(());
    }
    let (mut server_side, client_side_generator) = interaction_channel(1);

    let _ws = tokio::spawn(websocket_main(client_side_generator(
        &mut server_side,
        Clients::WebSocket,
    )));
    let _http = tokio::spawn(http_main(client_side_generator(
        &mut server_side,
        Clients::Http,
    )));

    while let Some(req) = server_side.recieve.recv().await {
        match req {
            ClientInteractions::WsSocket => server_side.respond(
                Clients::WebSocket,
                ServerInteractions::WsSocket(server.get_addr_websocket()),
            ),
            ClientInteractions::WsValidateClient(token) => server_side.respond(
                Clients::WebSocket,
                ServerInteractions::WsValidateClient(server.is_client_valid(token.as_str())),
            ),
            ClientInteractions::WsClientConnected { addr, client } => {
                server.client_connected(addr, client);
                server_side.respond(Clients::WebSocket, ServerInteractions::WsClientConnected)
            }
            ClientInteractions::WsGetConnectedClients => server_side.respond(
                Clients::WebSocket,
                ServerInteractions::WsGetConnectedClients(server.get_connected_clients()),
            ),
            ClientInteractions::WsSetClientConnectedTx { addr, tx } => {
                server.set_connected_client_tx(&addr, tx);
                server_side.respond(
                    Clients::WebSocket,
                    ServerInteractions::WsSetClientConnectedTx,
                );
            }
            ClientInteractions::WsClientLeft { addr } => {
                server.client_disconnected(&addr);
                server_side.respond(Clients::WebSocket, ServerInteractions::WsClientLeft);
            }

            ClientInteractions::HttpSocket => server_side.respond(
                Clients::Http,
                ServerInteractions::HttpSocket(server.get_addr_http()),
            ),

            ClientInteractions::HttpValidateClient(token) => server_side.respond(
                Clients::Http,
                ServerInteractions::HttpValidateClient(
                    server.is_client_connected_by_token(token.as_str()),
                ),
            ),

            ClientInteractions::HttpGetAllClients => server_side.respond(
                Clients::Http,
                ServerInteractions::HttpGetAllClients(server.get_all_clients()),
            ),
            ClientInteractions::HttpGetConnectedClients => server_side.respond(
                Clients::Http,
                ServerInteractions::HttpGetConnectedClients(server.get_connected_clients()),
            ),
        };
    }
    Ok(())
}
