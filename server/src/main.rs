use std::{collections::HashMap, error::Error};

use makar_protocol::*;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use log::{error, info};

pub async fn connection_task(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut players = HashMap::new();

    let mut size = [0u8; 4];
    loop {
        socket.read_exact(&mut size).await?;
        let size = u32::from_be_bytes(size);

        let mut buffer = vec![0u8; size as usize];
        socket.read_exact(&mut buffer).await?;

        let packet = ServerBoundPacket::deserialize(&buffer)?;
        match packet {
            ServerBoundPacket::JoinGameRequest { id, username } => {
                players.insert(id, username);

                let packet = ProxyBoundPacket::JoinGame {
                    player: id,
                    entity_id: 999,
                    gamemode: Gamemode::Survival,
                    dimension: 0,
                    difficulty: Difficulty::Easy,
                    max_players: 20,
                    level_type: "default".to_string(),
                    reduced_debug_info: false,
                }
                .serialize()?;
                socket.write_all(&packet).await?;
            }
            ServerBoundPacket::ClientSettings { player, locale } => {
                let message = match locale.as_str() {
                    "fr_FR" => "bonjour, bienvenue sur le serveur!",
                    _ => "hello, welcome to the server!",
                };

                let packet = ProxyBoundPacket::ChatMessage {
                    player,
                    json: Chat {
                        text: message.to_string(),
                        color: Some("blue".to_string()),
                        bold: None,
                        italic: None,
                        underlined: None,
                        strikethrough: None,
                        obfuscated: None,
                    },
                    position: 0,
                }
                .serialize()?;
                socket.write_all(&packet).await?;

                let message = match locale.as_str() {
                    "fr_FR" => "Salut!",
                    _ => "Hey there!",
                };

                let packet = ProxyBoundPacket::Title {
                    player,
                    action: TitleAction::Set {
                        title: Some(Chat {
                            text: message.to_string(),
                            color: Some("aqua".to_string()),
                            bold: None,
                            italic: None,
                            underlined: None,
                            strikethrough: None,
                            obfuscated: None,
                        }),
                        subtitle: None,
                        fade_in: 30,
                        stay: 1000,
                        fade_out: 30,
                    },
                }
                .serialize()?;
                socket.write_all(&packet).await?;
            }
            ServerBoundPacket::ChatMessage { player, message } => {
                let author = match players.get(&player) {
                    Some(v) => v,
                    None => continue,
                };
                for player in players.keys() {
                    let packet = ProxyBoundPacket::ChatMessage {
                        player: *player,
                        json: Chat {
                            text: format!("<{author}> {message}"),
                            color: None,
                            bold: None,
                            italic: None,
                            underlined: None,
                            strikethrough: None,
                            obfuscated: None,
                        },
                        position: 0,
                    }
                    .serialize()?;
                    socket.write_all(&packet).await?;
                }
            }
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let server = TcpListener::bind("127.0.0.1:25566").await?;
    info!("accepting connections on port 25566");

    loop {
        let (socket, _) = server.accept().await?;
        tokio::spawn(async move {
            match connection_task(socket).await {
                Ok(_) => {}
                Err(e) => {
                    error!("connection task ended unexpectingly: {e}");
                }
            };
        });
    }
}
