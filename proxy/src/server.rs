use std::error::Error;

use makar_protocol::{ProxyBoundPacket, ServerBoundPacket, TitleAction};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
};

use log::info;

use crate::versions::v1_8_8::*;

pub async fn server_task(
    address: &str,
    mut rx: mpsc::Receiver<ServerBoundPacket>,
    players: mpsc::Sender<crate::players::Message>,
) -> Result<(), Box<dyn Error>> {
    let mut socket = TcpStream::connect(address).await?;
    info!("connected to server at {address}");

    let mut size = [0u8; 4];
    loop {
        tokio::select! {
            _ = socket.read_exact(&mut size) => {
                let size = u32::from_be_bytes(size);
                let mut buf = vec![0u8; size as usize];
                socket.read_exact(&mut buf).await?;
                let packet = ProxyBoundPacket::deserialize(&buf)?;
                match packet {
                    ProxyBoundPacket::JoinGame { player, entity_id, gamemode, dimension, difficulty, max_players, level_type, reduced_debug_info } => {
                        let packet = ClientBoundPacket::JoinGame {
                            entity_id,
                            gamemode,
                            dimension,
                            difficulty,
                            max_players,
                            level_type,
                            reduced_debug_info: if reduced_debug_info { 1 } else { 0 },
                        };
                        players.send(crate::players::Message::Send(player, packet.serialize().to_vec())).await?;
                    }
                    ProxyBoundPacket::ChatMessage { player, json, position } => {
                        let packet = ClientBoundPacket::ChatMessage {
                            json: json.into(),
                            position,
                        };
                        players.send(crate::players::Message::Send(player, packet.serialize().to_vec())).await?;
                    }
                    ProxyBoundPacket::Title { player, action } => {
                        match action {
                            TitleAction::Set { title, subtitle, fade_in, stay, fade_out } => {
                                match title {
                                    Some(chat) => players.send(crate::players::Message::Send(player, ClientBoundPacket::Title { action: crate::protocol::TitleAction::SetTitle(chat.into()) }.serialize().to_vec())).await?,
                                    None => {},
                                };
                                match subtitle {
                                    Some(chat) => players.send(crate::players::Message::Send(player, ClientBoundPacket::Title { action: crate::protocol::TitleAction::SetSubtitle(chat.into()) }.serialize().to_vec())).await?,
                                    None => {},
                                };

                                players.send(crate::players::Message::Send(player, ClientBoundPacket::Title { action: crate::protocol::TitleAction::SetTimes { fade_in, stay, fade_out  } }.serialize().to_vec())).await?;
                            },
                            TitleAction::Hide => {
                                players.send(crate::players::Message::Send(player, ClientBoundPacket::Title { action: crate::protocol::TitleAction::Hide }.serialize().to_vec())).await?;
                            },
                            TitleAction::Reset => {
                                players.send(crate::players::Message::Send(player, ClientBoundPacket::Title { action: crate::protocol::TitleAction::Reset }.serialize().to_vec())).await?;
                            },
                        }
                    }
                };
            }
            msg = rx.recv() => {
                match msg {
                    Some(message) => {
                        socket.write_all(&message.serialize()?).await?;
                    }
                    None => {}
                };
            }
        }
    }
}
