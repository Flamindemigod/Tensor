use crate::{
    channel::{ClientChannel, ClientInteractions},
    message::{ClientSend, Message as ServerMessage},
};
use anyhow::Result;
use futures::executor::block_on;
use futures_channel::mpsc::unbounded;
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_tungstenite::tungstenite::{
    handshake::server::{Request, Response},
    http::{Response as http_Response, StatusCode},
};

async fn handle_connection(
    raw_stream: TcpStream,
    addr: SocketAddr,
    client_channel: Arc<Mutex<ClientChannel>>,
) {
    println!("Incoming TCP connection from: {}", addr);
    let callback = |req: &Request, mut response: Response| {
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
                    .body(Some("No Authorization Token ?Provided".to_owned()))
                    .unwrap());
            }
            Some(token) => {
                let auth = block_on(async {
                    let mut channel = client_channel.lock().await;
                    let client = channel
                        .request(ClientInteractions::WsValidateClient(token.clone()))
                        .await
                        .client_validation();
                    if let Some(client_inner) = client {
                        channel
                            .request(ClientInteractions::WsClientConnected {
                                addr,
                                client: client_inner,
                            })
                            .await;
                        drop(channel);
                        return true;
                    }
                    drop(channel);
                    false
                });
                if !auth {
                    return Err(http_Response::builder()
                        .status(StatusCode::NETWORK_AUTHENTICATION_REQUIRED)
                        .body(Some("No Invalid Token Provided".to_owned()))
                        .unwrap());
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

    let connected_clients = client_channel
        .lock()
        .await
        .request(ClientInteractions::WsGetConnectedClients)
        .await
        .connected_clients();

    let message = ServerMessage::new_server_message(format!(
        "<<!{}>> joined the server",
        connected_clients.unwrap().get(&addr).unwrap().get_uuid()
    ))
    .to_message();

    let (tx, rx) = unbounded();
    client_channel
        .lock()
        .await
        .request(ClientInteractions::WsSetClientConnectedTx { addr, tx })
        .await;
    let connected_clients = client_channel
        .lock()
        .await
        .request(ClientInteractions::WsGetConnectedClients)
        .await
        .connected_clients()
        .unwrap();

    connected_clients.values().for_each(|client| {
        let _ = client.tx.as_ref().unwrap().unbounded_send(message.clone());
    });
    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        if let Some(client_message) = ClientSend::parse(msg.clone().into_data()) {
            println!(
                "Received a message from {}: {}",
                addr, client_message.message
            );
            let peers = block_on(async {
                client_channel
                    .lock()
                    .await
                    .request(ClientInteractions::WsGetConnectedClients)
                    .await
                    .connected_clients()
                    .unwrap()
            });
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
    let peers = client_channel
        .lock()
        .await
        .request(ClientInteractions::WsGetConnectedClients)
        .await
        .connected_clients()
        .unwrap();

    let message = ServerMessage::new_server_message(format!(
        "<<!{}>> disconnected from the server",
        peers.get(&addr).unwrap().get_uuid()
    ))
    .to_message();
    client_channel
        .lock()
        .await
        .request(ClientInteractions::WsClientLeft { addr })
        .await;
    let peers = client_channel
        .lock()
        .await
        .request(ClientInteractions::WsGetConnectedClients)
        .await
        .connected_clients()
        .unwrap();
    peers.values().for_each(|client| {
        let _ = client.tx.as_ref().unwrap().unbounded_send(message.clone());
    });
}

pub async fn websocket_main(mut client: ClientChannel) -> Result<()> {
    let addr = client
        .request(ClientInteractions::WsSocket)
        .await
        .socket_addr()
        .unwrap();
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening to WebSocket Requests on: {}", addr);

    let client_channel = Arc::new(Mutex::new(client));
    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr, client_channel.clone()));
    }

    Ok(())
}
