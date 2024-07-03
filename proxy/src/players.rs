use std::{collections::HashMap, error::Error};

use tokio::sync::mpsc;

pub enum Message {
    Put(u128, mpsc::Sender<Vec<u8>>),
    Send(u128, Vec<u8>),
    Del(u128),
}

pub async fn players_task(mut rx: mpsc::Receiver<Message>) -> Result<(), Box<dyn Error>> {
    let mut players = HashMap::new();
    loop {
        match rx.recv().await {
            Some(Message::Put(id, channel)) => {
                players.insert(id, channel);
            }
            Some(Message::Send(id, data)) => match players.get(&id) {
                Some(tx) => {
                    tx.send(data).await?;
                }
                None => {}
            },
            Some(Message::Del(id)) => {
                players.remove(&id);
            }
            None => {}
        };
    }
}
