use std::error::Error;

use makar_protocol::{ProxyBoundPacket, ServerBoundPacket};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use log::info;

pub async fn connection_task(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut size = [0u8; 4];
    loop {
        socket.read_exact(&mut size).await?;
        let size = u32::from_be_bytes(size);

        let mut buffer = vec![0u8; size as usize];
        socket.read_exact(&mut buffer).await?;

        let packet = ServerBoundPacket::deserialize(&buffer)?;
        match packet {
            ServerBoundPacket::JoinGameRequest(player) => {
                let packet = ProxyBoundPacket::JoinGame {
                    player,
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
        tokio::spawn(async move { connection_task(socket).await.unwrap() });
    }
}
