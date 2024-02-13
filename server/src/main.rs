pub mod client;
pub mod message;
pub mod server;
use std::{
    net::SocketAddr,
    process::exit,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use futures_channel::mpsc::unbounded;
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use lazy_static::lazy_static;
use server::Server;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::{
    handshake::server::{Request, Response},
    http::{Response as http_Response, StatusCode},
    protocol::Message,
};

use crate::message::{ClientSend, Message as ServerMessage};

lazy_static! {
    pub static ref SERVER: Arc<Mutex<Server>> = Arc::from(Mutex::new(Server::init_server()));
}

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);
    let callback = |req: &Request, mut response: Response| {
        for (ref header, _value) in req.headers() {
            println!("* {}: {:?}", header, _value);
        }
        let protoheaders = req.headers().get("Sec-WebSocket-Protocol").unwrap();
        let binding = protoheaders
            .to_str()
            .unwrap()
            .split(", ")
            .map(|v| v.to_owned())
            .collect::<Vec<String>>();
        let mut protocol_segments = binding.iter();
        let token = match protocol_segments.next() {
            Some(t) => {
                if t.eq("Authorization") {
                    protocol_segments.next()
                } else {
                    None
                }
            }
            _ => None,
        };
        match token {
            None => {
                return Err(http_Response::builder()
                    .status(StatusCode::NETWORK_AUTHENTICATION_REQUIRED)
                    .body(Some("No Authorization Token Provided".to_owned()))
                    .unwrap());
            }
            Some(token) => {
                println!("Got Token from client header: {}", token.as_str());
                let client = SERVER.lock().unwrap().is_client_valid(token.as_str());
                match client {
                    Some(c) => SERVER.lock().unwrap().client_connected(addr, c),
                    None => {
                        return Err(http_Response::builder()
                            .status(StatusCode::NETWORK_AUTHENTICATION_REQUIRED)
                            .body(Some("No Invalid Token Provided".to_owned()))
                            .unwrap());
                    }
                }
            }
        }
        println!("protocol_segments: {protocol_segments:#?}");

        let headers = response.headers_mut();
        headers.insert("Sec-Websocket-Protocol", "Authorization".parse().unwrap());

        Ok(response)
    };

    let ws_stream = tokio_tungstenite::accept_hdr_async(raw_stream, callback)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("WebSocket connection established: {}", addr);

    let (tx, rx) = unbounded();
    SERVER.lock().unwrap().set_connected_client_tx(&addr, tx);

    let (outgoing, incoming) = ws_stream.split();
    let broadcast_incoming = incoming.try_for_each(|msg| {
        if let Some(client_message) = ClientSend::parse(msg.clone().into_data()) {
            println!(
                "Received a message from {}: {}",
                addr, client_message.message
            );
            let peers = SERVER.lock().unwrap().get_connected_clients();
            // We want to broadcast the message to everyone except ourselves.
            let broadcast_recipients = peers
                .iter()
                .filter(|(peer_addr, _)| peer_addr != &&addr)
                .map(|(_, client)| client);
            let sender = peers
                .iter()
                .filter(|(peer_addr, _)| peer_addr == &&addr)
                .map(|(_, client)| client.get_uuid())
                .next()
                .unwrap();
            let server_message = ServerMessage::new(client_message.message, sender);
            let mentions = server_message.mentions();

            for recp in broadcast_recipients {
                let m = if mentions.contains(&recp.get_uuid().to_string()) {
                    server_message.clone().set_mention()
                } else {
                    server_message.clone()
                };
                recp.tx
                    .as_ref()
                    .unwrap()
                    .unbounded_send(Message::from(format!("{:#?}", m)))
                    .expect("Failed to Send Message to Peers");
            }
            future::ok(())
        } else {
            // let stream = (incoming.reunite(outgoing)).unwrap();
            // drop(stream);
            //SERVER.lock().unwrap().client_disconnected(&addr);
            future::err(tokio_tungstenite::tungstenite::Error::ConnectionClosed)
        }
    });
    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;
    println!("{} disconnected", &addr);
    SERVER.lock().unwrap().client_disconnected(&addr);
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SERVER.lock().unwrap().get_addr();
    let _ = ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        SERVER.lock().unwrap().cleanup();
        exit(0)
    });

    // SERVER.lock().unwrap().new_client("Veltearas");

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }
    Ok(())
}
