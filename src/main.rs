pub mod channel;
pub mod client;
pub mod http;
pub mod message;
pub mod server;
pub mod websocket;

use anyhow::Result;
use channel::{interaction_channel, ClientInteractions, Clients, ServerInteractions};
use std::process::exit;

use http::http_main;
use server::Server;
use websocket::websocket_main;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        // SERVER.read().unwrap().cleanup();
        exit(0)
    });

    //Init Server:
    let mut server = Server::init_server();
    let (mut server_side, client_side_generator) = interaction_channel(1);
    // SERVER.lock().unwrap().new_client("Flamindemigod");
    let _ws = tokio::spawn(websocket_main(client_side_generator(
        &mut server_side,
        Clients::WebSocket,
    )));
    let _http = tokio::spawn(http_main(client_side_generator(
        &mut server_side,
        Clients::Http,
    )));
    // let http = http_main();
    // let _ = join!(
    //     ws,
    //     // http
    //     );
    //
    // let _ = ws.;
    // let _ = join!(ws);
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
