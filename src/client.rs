//File Contains Structs for Client Represententaion and Manipulation

use rand::distributions::{Alphanumeric, DistString};
use serde::Serialize;
use sqlite::{Connection, Row, Value};
use std::sync::Arc;
use derivative::Derivative;
use crate::server::Tx;
#[derive(Derivative, Serialize)]
#[derivative(Debug, Clone,Hash, PartialEq, Eq)]
pub struct Client {
    uuid: Arc<str>,
    #[serde(skip)]
    token: Arc<str>,

    pub username: String,
    pub display_name: String,
    pub about_me: String,
    #[derivative(PartialEq="ignore")]
    #[derivative(Hash="ignore")]
    #[serde(skip)]
    pub tx: Option<Tx>,
}

impl Client {
    // Add new Client
    fn generate_token() -> Arc<str> {
        let s = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        Arc::from(s.as_str())
    }

    fn generate_uuid() -> Arc<str> {
        let mut s = "".to_string();
        for _ in 0..4 {
            s.push_str(&Alphanumeric.sample_string(&mut rand::thread_rng(), 3));
            s.push('-')
        }
        Arc::from(s.as_str())
    }

    pub fn from_db_row(row: Row) -> Self {
        Self {
            uuid: row.read::<&str, _>("uuid").into(),
            token: row.read::<&str, _>("token").into(),
            username: row.read::<&str, _>("username").into(),
            display_name: row.read::<&str, _>("display_name").into(),
            about_me: row.read::<&str, _>("about_me").into(),
            tx: None,
        }
    }

    pub fn new(username: &str, connection: &Connection) -> Self {
        let s = Self {
            uuid: Self::generate_uuid(),
            token: Self::generate_token(),
            username: username.to_string(),
            display_name: username.to_string(),
            about_me: String::new(),
            tx: None,
        };
        s.write_to_db(connection);
        s
    }

    pub fn regenerate_token(&mut self) {
        self.token = Self::generate_token();
    }

    pub fn get_token(&self) -> Arc<str> {
        self.token.clone()
    }
    pub fn get_uuid(&self) -> Arc<str> {
        self.uuid.clone()
    }

    pub fn write_to_db(&self, connection: &Connection) {
        //"CREATE TABLE clients (uuid TEXT, token TEXT, username TEXT, display_name TEXT, about_me TEXT);";
        let query = "INSERT INTO clients VALUES (?, ?, ?, ?, ?)";
        let mut statement = connection.prepare(query).unwrap();
        statement
            .bind_iter::<_, (_, Value)>([
                (1, self.uuid.to_string().as_str().into()),
                (2, self.token.to_string().as_str().into()),
                (3, self.username.clone().into()),
                (4, self.display_name.clone().into()),
                (5, self.about_me.clone().into()),
            ])
            .unwrap();
        let _ = statement.next();
    }
}
