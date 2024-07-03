use std::error::Error;

use makar_protocol::{ProxyBoundPacket, ServerBoundPacket};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
};

pub async fn server_task(
    address: &str,
    mut rx: mpsc::Receiver<ServerBoundPacket>,
    players: mpsc::Sender<crate::players::Message>,
) -> Result<(), Box<dyn Error>> {
    let mut socket = TcpStream::connect(address).await?;
    println!("connected to {address}");
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
                        let packet = crate::versions::v1_8_8::JoinGame {
                            entity_id,
                            gamemode,
                            dimension,
                            difficulty,
                            max_players,
                            level_type,
                            reduced_debug_info: if reduced_debug_info { 1 } else { 0 },
                        };
                        println!("proxy -> player ({packet:?})");
                        players.send(crate::players::Message::Send(player, packet.serialize().to_vec())).await?;
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
