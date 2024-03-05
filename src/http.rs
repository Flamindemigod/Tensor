use std::{collections::HashSet, net::SocketAddr, sync::Arc};

use anyhow::Result;
use futures::executor::block_on;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{
    body::Bytes, header::HeaderValue, server::conn::http1::Builder, service::service_fn, HeaderMap,
    Method, Request, Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use serde::Serialize;
use serde_json::Map;
use tokio::{net::TcpListener, sync::Mutex};

use crate::channel::{ClientChannel, ClientInteractions};

pub fn json_bytes<T>(structure: T) -> Vec<u8>
where
    T: Serialize,
{
    let mut bytes: Vec<u8> = Vec::new();
    serde_json::to_writer(&mut bytes, &structure).unwrap();
    bytes
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

async fn is_auth(
    req: &Request<impl hyper::body::Body>,
    client_channel: Arc<Mutex<ClientChannel>>,
) -> bool {
    req.headers()
        .get("authorization")
        .map(|f| {
            if block_on(async {
                client_channel
                    .lock()
                    .await
                    .request(ClientInteractions::HttpValidateClient(
                        f.to_str().unwrap().to_string(),
                    ))
                    .await
                    .get_bool()
            }) {
                return true;
            }
            false
        })
        .unwrap_or(false)
}

pub async fn preflight(
    _: Request<impl hyper::body::Body>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let mut res = Response::new(empty());
    *res.status_mut() = StatusCode::OK;
    let mut headers = HeaderMap::new();
    headers.insert(
        "Access-Control-Allow-Origin",
        HeaderValue::from_str("*").unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        HeaderValue::from_str("*").unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_str("GET, POST, OPTIONS").unwrap(),
    );
    *res.headers_mut() = headers;
    Ok(res)
}

async fn handle_request(
    req: Request<impl hyper::body::Body>,
    _addr: SocketAddr,
    client_channel: Arc<Mutex<ClientChannel>>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    if req.method() == Method::OPTIONS {
        return preflight(req).await;
    };
    if !is_auth(&req, client_channel.clone()).await {
        let mut rej = Response::new(full(Bytes::from("UNAUTHORIZED\n")));
        *rej.status_mut() = StatusCode::UNAUTHORIZED;
        return Ok(rej);
    }
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/list_clients") => {
            let all_clients = client_channel
                .lock()
                .await
                .request(ClientInteractions::HttpGetAllClients)
                .await
                .all_clients();
            let binding = client_channel
                .lock()
                .await
                .request(ClientInteractions::HttpGetConnectedClients)
                .await
                .connected_clients()
                .unwrap();
            let connected_clients = binding.values().collect::<HashSet<_>>();
            let mut map = Map::new();
            map.insert(
                "offline".to_string(),
                serde_json::to_value(
                    all_clients
                        .iter()
                        .filter(|c| !connected_clients.contains(c))
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            );
            map.insert(
                "online".to_string(),
                serde_json::to_value(connected_clients).unwrap(),
            );
            let mut res = Response::new(full(json_bytes(serde_json::to_value(map).unwrap())));
            let mut headers = HeaderMap::new();
            headers.insert(
                "Access-Control-Allow-Origin",
                HeaderValue::from_str("*").unwrap(),
            );
            headers.insert(
                "Access-Control-Allow-Headers",
                HeaderValue::from_str("*").unwrap(),
            );
            headers.insert(
                "Access-Control-Allow-Methods",
                HeaderValue::from_str("GET, POST, OPTIONS").unwrap(),
            );
            *res.headers_mut() = headers;
            Ok(res)
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(full(Bytes::from("404: Not Found\n")))
            .unwrap()),
    }
}

pub async fn http_main(mut client: ClientChannel) -> Result<()> {
    let addr = client
        .request(ClientInteractions::HttpSocket)
        .await
        .socket_addr()
        .unwrap();

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening to HTTP Requests on: {}", addr);

    let client_channel = Arc::new(Mutex::new(client));

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        let req_wrapper = |req| handle_request(req, addr, client_channel.clone());
        let io = TokioIo::new(stream);
        let _ = Builder::new()
            .serve_connection(io, service_fn(req_wrapper))
            .await;
    }
    Ok(())
}
