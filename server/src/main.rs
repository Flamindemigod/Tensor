pub mod client;
pub mod message;
pub mod server;
pub mod websocket;
pub mod http;

use std::{
    process::exit,
    sync::{Arc, Mutex},
};
use anyhow::Result;

use futures_util::join;
use lazy_static::lazy_static;
use server::Server;
use websocket::websocket_main;
use http::http_main;


lazy_static! {
    pub static ref SERVER: Arc<Mutex<Server>> = Arc::from(Mutex::new(Server::init_server()));
}


#[tokio::main]
async fn main() -> Result<()> {
    let _ = ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        SERVER.lock().unwrap().cleanup();
        exit(0)
    });

    // SERVER.lock().unwrap().new_client("Flamindemigod");
    let ws = websocket_main();
    let http = http_main();
    let _  = join!(ws, http); 
    Ok(())
}
