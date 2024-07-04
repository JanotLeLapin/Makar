use std::error::Error;

use makar_protocol::{ProxyBoundPacket, ServerBoundPacket};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use log::{error, info};

pub async fn connection_task(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut size = [0u8; 4];
    loop {
        socket.read_exact(&mut size).await?;
        let size = u32::from_be_bytes(size);

        let mut buffer = vec![0u8; size as usize];
        socket.read_exact(&mut buffer).await?;

        let packet = ServerBoundPacket::deserialize(&buffer)?;
        match packet {
            ServerBoundPacket::JoinGameRequest { id, username } => {
                let packet = ProxyBoundPacket::JoinGame {
                    player: id,
                    entity_id: 999,
                    gamemode: 0,
                    dimension: 0,
                    difficulty: 0,
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
                    json: format!("{{\"text\":\"{message}\",\"color\":\"blue\"}}"),
                    position: 0,
                }
                .serialize()?;
                socket.write_all(&packet).await?;
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
