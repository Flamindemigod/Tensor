use std::sync::Arc;

use rand::distributions::{Alphanumeric, DistString};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use tokio_tungstenite::tungstenite::protocol::Message as Message_Tungestenite;

//Server Response to Peers
#[derive(Clone, Debug, Serialize)]
pub struct Message {
    message_uuid: Arc<str>,
    author_uuid: Arc<str>,
    data: String,
    edited: bool,
    pub is_mentioned: bool,
}

impl Message {
    fn generate_message_id() -> Arc<str> {
        Arc::from(Alphanumeric.sample_string(&mut rand::thread_rng(), 16))
    }

    pub fn new(data: String, author_uuid: Arc<str>) -> Self {
        Self {
            message_uuid: Self::generate_message_id(),
            author_uuid,
            data,
            edited: false,
            is_mentioned: false,
        }
    }

    pub fn mentions(&self) -> Vec<String> {
        let re = Regex::new(r"<<!(.{16})>>").unwrap();
        let mut mentions = vec![];
        for (_, [uuid]) in re.captures_iter(&self.data).map(|c| c.extract()) {
            mentions.push(uuid.to_string());
        }
        mentions
    }

    pub fn set_mention(&self) -> Self {
        let mut new_message = self.clone();
        new_message.is_mentioned = true;
        new_message
    }

    pub fn to_message(&self) -> Message_Tungestenite{
        Message_Tungestenite::from(serde_json::to_string(self).unwrap())
    }
}

#[derive(Debug, Clone, Copy, Deserialize_repr)]
#[repr(u8)]
pub enum MessageOps {
    NewMessage,
    EditMessage,
    DeleteMessage,
}

//Client Message recieve
#[derive(Debug, Clone, Deserialize)]
pub struct ClientSend {
    pub op: MessageOps,
    pub allowed_mentions: bool,
    message_uuid: Option<String>,
    pub message: String,
}

impl ClientSend {
    pub fn parse(data: Vec<u8>) -> Option<Self> {
        // println!("Parsing Json String {:#?}", data);
        if let Ok(mut k) = serde_json::from_slice::<Self>(data.as_slice()) {
            k.parse_message_uuid();
            match k.op {
                MessageOps::NewMessage => Some(k),
                MessageOps::EditMessage | MessageOps::DeleteMessage => {
                    if k.message_uuid.is_some() {
                        Some(k)
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn parse_message_uuid(&mut self) {
        match self.op {
            MessageOps::NewMessage => {
                self.message_uuid = None;
            }
            MessageOps::EditMessage | MessageOps::DeleteMessage => {
                if let Some(uuid) = self
                    .message_uuid
                    .as_ref()
                    .map(|uuid| uuid.chars().take(16).collect::<String>())
                {
                    if uuid.len() == 16 {
                        self.message_uuid = Some(uuid)
                    } else {
                        self.message_uuid = None;
                    }
                }
            }
        }
    }
}
