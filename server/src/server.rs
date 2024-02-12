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
    server_addr: SocketAddr,
    server_name: Arc<str>,
    client_token: Arc<str>,
}
impl ClientExport {
    pub fn new(server: &Server, client: &Client) -> Self {
        let e = Self {
            server_addr: server.get_addr(),
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
    server_port: u16,
    pub export_path: Option<PathBuf>,
    pub db_path: PathBuf,
    #[serde(skip)]
    db: Option<Connection>,
    #[serde(skip)]
    connected_clients: HashMap<SocketAddr, Client>,
}

impl Server {
    fn set_db_connection(&mut self) {
        let new_db = !self.db_path.exists();
        self.db = Some(sqlite::open(&self.db_path).unwrap_or_else(|_| {
            panic!(
                "Failed to Open Connection to db at {}",
                &self.db_path.display()
            )
        }));
        if new_db {
            let query = "CREATE TABLE clients (uuid TEXT, token TEXT, username TEXT, display_name TEXT, about_me TEXT);";
            self.db
                .as_mut()
                .unwrap()
                .execute(query)
                .expect("Failed to Create Table");
        }
    }

    pub fn init_server() -> Self {
        let mut reader = File::open("./config.json").expect("Failed to Open Server Config File");
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .expect("Failed to Read File");
        let mut server = serde_json::from_str::<Self>(&buf).expect("Failed to Parse Server Config");
        server.set_db_connection();
        server
    }

    pub fn cleanup(&mut self) {
        self.db = None;
    }

    pub fn get_addr(&self) -> SocketAddr {
        SocketAddr::new(self.server_ip, self.server_port)
    }

    pub fn is_client_valid(&self, token: &str) -> Option<Client> {
        let query = "SELECT * FROM clients WHERE token = ?";
        self.db
            .as_ref()
            .expect("Connection does not exist")
            .prepare(query)
            .unwrap()
            .into_iter()
            .bind((1, token))
            .unwrap()
            .map(|row| Client::from_db_row(row.unwrap()))
            .next()
    }

    pub fn client_connected(&mut self, addr: SocketAddr, client: Client) {
        if self.is_client_valid(&client.get_token()).is_some() {
            self.connected_clients.insert(addr, client);
        }
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

    pub fn new_client(&self, username: &str) {
        let client = Client::new(username, self.db.as_ref().unwrap());
        ClientExport::new(self, &client);
    }
}
