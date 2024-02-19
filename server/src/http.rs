use std::collections::HashSet;

use crate::SERVER;
use anyhow::Result;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{
    body::Bytes, server::conn::http1::Builder, service::service_fn, Method, Request, Response, StatusCode
};
use hyper_util::rt::TokioIo;
use serde::Serialize;
use serde_json::Map;
use tokio::net::TcpListener;

pub fn json_bytes<T>(structure: T) -> Vec<u8> where T: Serialize {
    let mut bytes: Vec<u8> = Vec::new();
    serde_json::to_writer(&mut bytes, &structure).unwrap();
    bytes
}

// fn empty() -> BoxBody<Bytes, hyper::Error> {
//     Empty::<Bytes>::new()
//         .map_err(|never| match never {})
//         .boxed()
// }

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
async fn is_auth(req: &Request<impl hyper::body::Body>) -> bool {
    req.headers()
        .get("authorization")
        .map(|f| {
            if SERVER.lock().unwrap().is_client_connected_by_token(f.to_str().unwrap()) {
                return true;
            }
            false
        })
        .unwrap_or(false)
}

async fn handle_request(
    req: Request<impl hyper::body::Body>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    if !is_auth(&req).await {
        let mut rej = Response::new(full(Bytes::from("UNAUTHORIZED\n")));
        *rej.status_mut() = StatusCode::UNAUTHORIZED;
        return Ok(rej);
    }
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/list_clients") =>  {
            let all_clients = SERVER.lock().unwrap().get_all_clients();
            let binding = SERVER.lock().unwrap().get_connected_clients();
            let connected_clients = binding.values().collect::<HashSet<_>>();
            let mut map = Map::new();
            map.insert("offline".to_string(), serde_json::to_value(all_clients.iter().filter(|c| !connected_clients.contains(c)).collect::<Vec<_>>()).unwrap());
            map.insert("online".to_string(), serde_json::to_value(connected_clients).unwrap());
            println!("all clients: {map:#?}");
            Ok(Response::new(full(json_bytes(serde_json::to_value(map).unwrap()))))
        },
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(full(Bytes::from("404: Not Found\n")))
            .unwrap()),
    }
}

pub async fn http_main() -> Result<()> {
    let addr = SERVER.lock().unwrap().get_addr_http();
    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening to HTTP Requests on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, _addr)) = listener.accept().await {
        let io = TokioIo::new(stream);
        let _ = Builder::new().serve_connection(io, service_fn(handle_request)).await;
    }

    Ok(())
}
