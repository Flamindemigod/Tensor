// File Contains Structs for Server Config and Manipulation

use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};

use futures_channel::mpsc::UnboundedSender;
use serde::{Deserialize, Serialize};
use sqlite::Connection;
use tokio_tungstenite::tungstenite::Message;

use crate::client::Client;

pub type Tx = UnboundedSender<Message>;

#[derive(Serialize)]
struct ClientExport {
    server_ip: IpAddr,
    websocket_server_port: u16,
    http_server_port: u16,
    server_name: Arc<str>,
    client_token: Arc<str>,
}
impl ClientExport {
    pub fn new(server: &Server, client: &Client) -> Self {
        let addr = server.get_addr_websocket();
        let e = Self {
            server_ip: addr.ip(),
            websocket_server_port: addr.port(),
            http_server_port: server.get_http_port(),
            server_name: server.server_name.clone(),
            client_token: client.get_token(),
        };
        e.export(
            format!("{:}-{:}", server.server_name.clone(), client.username).as_str(),
            server.export_path.clone(),
        );
        e
    }

    pub fn export(&self, file_name: &str, filepath: Option<PathBuf>) {
        let path = filepath.unwrap_or(PathBuf::from("."));
        let _ = std::fs::create_dir_all(&path);
        let mut writter = File::create(path.join(format!("{}.conf", file_name)))
            .map_err(|e| eprintln!("Error: Failed to open file to write client data: {e}"))
            .unwrap();
        let _ = writter
            .write_all(
                serde_json::to_string_pretty(self)
                    .expect("Failed to Convert Client Data to Config")
                    .as_bytes(),
            )
            .map_err(|e| eprintln!("Failed to Write Config: {e}"));
    }
}

#[derive(Deserialize)]
pub struct Server {
    pub server_name: Arc<str>,
    server_ip: IpAddr,
    websocket_server_port: u16,
    http_server_port: u16,
    pub export_path: Option<PathBuf>,
    pub db_path: PathBuf,
    #[serde(skip)]
    db_connection: Option<Connection>,
    #[serde(skip)]
    connected_clients: HashMap<SocketAddr, Client>,
}

impl Server {
    pub fn init_server() -> Self {
        let mut reader = File::open("./config.json").expect("Failed to Open Server Config File");
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .expect("Failed to Read File");
        let mut s = serde_json::from_str::<Self>(&buf).expect("Failed to Parse Server Config");

        let new_db = !s.db_path.exists();
        let db = sqlite::open(&s.db_path).unwrap_or_else(|_| {
            panic!("Failed to Open Connection to db at {}", s.db_path.display())
        });
        if new_db {
            let query = "CREATE TABLE clients (uuid TEXT, token TEXT, username TEXT, display_name TEXT, about_me TEXT);";
            db.execute(query).expect("Failed to Create Table");
        }
        s.db_connection = Some(db);

        s
    }

    pub fn cleanup(&self) {}

    pub fn get_websocket_port(&self) -> u16 {
        self.websocket_server_port
    }
    pub fn get_http_port(&self) -> u16 {
        self.http_server_port
    }
    pub fn get_addr_websocket(&self) -> SocketAddr {
        SocketAddr::new(self.server_ip, self.websocket_server_port)
    }

    pub fn get_addr_http(&self) -> SocketAddr {
        SocketAddr::new(self.server_ip, self.http_server_port)
    }

    pub fn is_client_valid(&mut self, token: &str) -> Option<Client> {
        let query = "SELECT * FROM clients WHERE token = ?";
        self.db_connection
            .as_mut()
            .unwrap()
            .prepare(query)
            .unwrap()
            .into_iter()
            .bind((1, token))
            .unwrap()
            .map(|row| Client::from_db_row(row.unwrap()))
            .next()
    }

    pub fn client_connected(&mut self, addr: SocketAddr, client: Client) {
        self.connected_clients.insert(addr, client);
    }
    pub fn set_connected_client_tx(&mut self, addr: &SocketAddr, tx: Tx) {
        self.connected_clients.get_mut(addr).unwrap().tx = Some(tx);
    }

    pub fn client_disconnected(&mut self, addr: &SocketAddr) {
        self.connected_clients.remove(addr);
    }

    pub fn get_connected_clients(&self) -> HashMap<SocketAddr, Client> {
        self.connected_clients.clone()
    }

    pub fn is_client_connected_by_token(&self, token: &str) -> bool {
        self.connected_clients
            .values()
            .any(|c| c.get_token() == token.into())
    }

    pub fn new_client(&mut self, username: &str) {
        let client = Client::new(username, self.db_connection.as_mut().unwrap());
        ClientExport::new(self, &client);
    }

    pub fn get_all_clients(&mut self) -> Vec<Client> {
        let query = "SELECT * FROM clients";
        self.db_connection
            .as_mut()
            .unwrap()
            .prepare(query)
            .unwrap()
            .into_iter()
            .map(|row| Client::from_db_row(row.unwrap()))
            .collect::<Vec<_>>()
    }
}
