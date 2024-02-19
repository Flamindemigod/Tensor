use crate::{
    message::{ClientSend, Message as ServerMessage},
    SERVER,
};
use anyhow::Result;
use futures_channel::mpsc::unbounded;
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::{
    handshake::server::{Request, Response},
    http::{Response as http_Response, StatusCode},
};

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
        let headers = response.headers_mut();
        headers.insert("Sec-Websocket-Protocol", "Authorization".parse().unwrap());
        Ok(response)
    };

    let ws_stream = tokio_tungstenite::accept_hdr_async(raw_stream, callback)
        .await
        .expect("Error during the websocket handshake occurred");

    let message = ServerMessage::new_server_message(format!(
        "<<!{}>> joined the server",
        SERVER
            .lock()
            .unwrap()
            .get_connected_clients()
            .get(&addr)
            .unwrap()
            .get_uuid()
    ))
    .to_message();
    let (tx, rx) = unbounded();
    SERVER.lock().unwrap().set_connected_client_tx(&addr, tx);
    SERVER
        .lock()
        .unwrap()
        .get_connected_clients()
        .values()
        .for_each(|client| {
            let _ = client.tx.as_ref().unwrap().unbounded_send(message.clone());
        });
    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        if let Some(client_message) = ClientSend::parse(msg.clone().into_data()) {
            println!(
                "Received a message from {}: {}",
                addr, client_message.message
            );
            let peers = SERVER.lock().unwrap().get_connected_clients();
            let broadcast_recipients = peers.values();
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
                    .unbounded_send(m.to_message())
                    .expect("Failed to Send Message to Peers");
            }
            future::ok(())
        } else {
            future::err(tokio_tungstenite::tungstenite::Error::ConnectionClosed)
        }
    });
    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;
    println!("{} disconnected", &addr);
    let peers = SERVER.lock().unwrap().get_connected_clients();

    let message = ServerMessage::new_server_message(format!(
        "<<!{}>> disconnected from the server",
        peers.get(&addr).unwrap().get_uuid()
    ))
    .to_message();
    SERVER.lock().unwrap().client_disconnected(&addr);
    let peers = SERVER.lock().unwrap().get_connected_clients();

    peers.values().for_each(|client| {
        let _ = client.tx.as_ref().unwrap().unbounded_send(message.clone());
    });
}

pub async fn websocket_main() -> Result<()> {
    let addr = SERVER.lock().unwrap().get_addr_websocket();
    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening to WebSocket Requests on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }

    Ok(())
}
