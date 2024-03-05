use std::{collections::HashMap, net::SocketAddr};

use tokio::sync::mpsc::{
    channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender,
};

use crate::{client::Client, server::Tx};
// use futures_util::StreamExt;
#[derive(Hash, PartialEq, Eq)]
pub enum Clients {
    WebSocket,
    Http,
}

//Requests to Server
pub enum ClientInteractions {
    WsSocket,
    WsValidateClient(String),
    WsClientConnected { addr: SocketAddr, client: Client },
    WsSetClientConnectedTx { addr: SocketAddr, tx: Tx },
    WsGetConnectedClients,
    WsClientLeft { addr: SocketAddr },

    HttpSocket,
    HttpValidateClient(String),
    HttpGetConnectedClients,
    HttpGetAllClients,
}

// Responses from Server
#[derive(Debug, Clone)]
pub enum ServerInteractions {
    WsSocket(SocketAddr),
    WsValidateClient(Option<Client>),
    WsClientConnected,
    WsSetClientConnectedTx,
    WsGetConnectedClients(HashMap<SocketAddr, Client>),
    WsClientLeft,

    HttpSocket(SocketAddr),
    HttpValidateClient(bool),
    HttpGetConnectedClients(HashMap<SocketAddr, Client>),
    HttpGetAllClients(Vec<Client>),
}

impl ServerInteractions {
    pub fn socket_addr(&self) -> Option<SocketAddr> {
        match self {
            Self::WsSocket(addr) => Some(*addr),
            Self::HttpSocket(addr) => Some(*addr),
            _ => None,
        }
    }
    pub fn client_validation(&self) -> Option<Client> {
        match self {
            Self::WsValidateClient(client) => client.clone(),
            _ => None,
        }
    }
    pub fn connected_clients(&self) -> Option<HashMap<SocketAddr, Client>> {
        match self {
            Self::WsGetConnectedClients(map) => Some(map.clone()),
            Self::HttpGetConnectedClients(map) => Some(map.clone()),
            _ => None,
        }
    }
    pub fn all_clients(&self) -> Vec<Client> {
        match self {
            Self::HttpGetAllClients(val) => val.to_owned(),
            _ => vec![],
        }
    }
    pub fn get_bool(&self) -> bool {
        match self {
            Self::HttpValidateClient(value) => *value,
            _ => false,
        }
    }
}

pub struct ServerChannel {
    pub send: HashMap<Clients, UnboundedSender<ServerInteractions>>,
    pub recieve: Receiver<ClientInteractions>,
}

pub struct ClientChannel {
    pub send: Sender<ClientInteractions>,
    pub recieve: UnboundedReceiver<ServerInteractions>,
}

impl ClientChannel {
    pub async fn request(&mut self, req: ClientInteractions) -> ServerInteractions {
        let _ = self
            .send
            .send(req)
            .await
            .map_err(|e| eprintln!("Failed to Send message to server : {e}"));
        self.recieve.recv().await.unwrap()
    }
}

impl ServerChannel {
    pub fn respond(&self, client: Clients, res: ServerInteractions) {
        let _ = self.send.get(&client).unwrap().send(res);
    }
}

type ClientChannelBuilder = Box<dyn for<'a> Fn(&'a mut ServerChannel, Clients) -> ClientChannel>;
pub fn interaction_channel(size: usize) -> (ServerChannel, ClientChannelBuilder) {
    // Server channel
    // Server           Client
    // Receiver         Sender <- Cloned
    // Sender           Receiver

    let server_reserved_channel = channel::<ClientInteractions>(size);
    // let b_c = channel(size);

    (
        ServerChannel {
            send: HashMap::new(),
            recieve: server_reserved_channel.1,
        },
        Box::new(move |server_side: &mut ServerChannel, client: Clients| {
            let b_c = unbounded_channel::<ServerInteractions>();
            server_side.send.insert(client, b_c.0);

            ClientChannel {
                recieve: b_c.1,
                send: server_reserved_channel.0.clone(),
            }
        }),
    )
}
