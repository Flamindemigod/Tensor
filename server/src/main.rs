use std::{
    collections::HashMap, env,  net::SocketAddr, sync::{Arc, Mutex}
};

use anyhow::Result;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::{handshake::server::{Request, Response}, http::{Response as http_Response, StatusCode}, protocol::Message};

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;


async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);


     let callback = |req: &Request, mut response: Response| {
        for (ref header, _value) in req.headers() {
            println!("* {}: {:?}", header, _value);
        }
        let protoheaders=  req.headers().get("Sec-WebSocket-Protocol").unwrap();
        let binding = protoheaders.to_str().unwrap().split(", ").map(|v| v.to_owned()).collect::<Vec<String>>();
        let mut protocol_segments = binding.iter();
        let token = match protocol_segments.next() {
        Some(t) => {if t.eq("Authorization") {protocol_segments.next()} else {None}},
            _ => None,
     };

        match token {
            None => return Err(http_Response::builder().status(StatusCode::NETWORK_AUTHENTICATION_REQUIRED).body(Some("No Authorization Token Provided".to_owned())).unwrap()),
            Some(token) => println!("Got Token from client header: {}", token.as_str())
        }
        println!("protocol_segments: {protocol_segments:#?}");
      
        let headers = response.headers_mut();
        headers.insert("Sec-Websocket-Protocol","Authorization".parse().unwrap());

        Ok(response)
    };

    let ws_stream = tokio_tungstenite::accept_hdr_async(raw_stream, callback)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        println!("Received a message from {}: {}", addr, msg.to_text().unwrap());
        let peers = peer_map.lock().unwrap();

        // We want to broadcast the message to everyone except ourselves.
        let broadcast_recipients =
            peers.iter().filter(|(peer_addr, _)| peer_addr != &&addr).map(|(_, ws_sink)| ws_sink);

        for recp in broadcast_recipients {
            recp.unbounded_send(Message::from(format!("{addr}: {}", msg.to_text().unwrap()))).unwrap();
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
}

 

#[tokio::main]
async fn main() -> Result<()> {
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, addr));
    }
    

    Ok(())
}
