use std::error::Error;

use bytes::{BufMut, Bytes, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
};

use log::{debug, info};

use crate::protocol::State;
use crate::versions::v1_8_8::*;

pub struct Player {
    pub state: State,
    pub protocol: Option<u16>,
    pub id: Option<u128>,
    pub username: Option<String>,
}

pub async fn connection_task(
    mut socket: TcpStream,
    mut rx: mpsc::Receiver<Vec<u8>>,
    tx: mpsc::Sender<Vec<u8>>,
    players: mpsc::Sender<crate::players::Message>,
    server: mpsc::Sender<makar_protocol::ServerBoundPacket>,
) -> Result<(), Box<dyn Error>> {
    let mut data = Player {
        state: State::Handshake,
        protocol: None,
        id: None,
        username: None,
    };

    loop {
        tokio::select! {
            b = socket.read_u8() => {
                let size = {
                    let mut res: i32 = 0;
                    let mut pos: u8 = 0;
                    let mut b = match b {
                        Ok(b) => b,
                        Err(_) => {
                            match data {
                                Player { id: Some(id), username: Some(username), .. } => {
                                    players.send(crate::players::Message::Del(id)).await?;
                                    info!("player {username} disconnected");
                                }
                                _ => {},
                            };
                            return Ok(());
                        }
                    };

                    loop {
                        res |= (b as i32 & 0x7F) << pos;
                        if (b & 0x80) == 0 {
                            break Some(res);
                        }

                        pos += 7;
                        if pos >= 32 {
                            break None;
                        }
                        b = socket.read_u8().await?;
                    }
                }.expect("Packet size too big");
                let mut buf = vec![0u8; size as usize];
                socket.read_exact(&mut buf).await?;
                // debug!("got {buf:?}");

                let bytes = Bytes::from(buf);
                let packet = ProxyBoundPacket::deserialize(&data.state, bytes)?;
                // debug!("{packet:?}");

                match packet {
                    ProxyBoundPacket::Handshake { protocol, next_state, .. } => {
                        data.protocol = Some(protocol.value() as u16);
                        data.state = match next_state {
                            1 => State::Status,
                            2 => State::Login,
                            v => return Err(format!("unknown state {v}").into()),
                        };
                    },
                    ProxyBoundPacket::StatusRequest {} => {
                        let (tx, rx) = tokio::sync::oneshot::channel();
                        players.send(crate::players::Message::Count(tx)).await?;
                        let count = rx.await?;
                        let status = format!("{{\"version\":{{\"name\":\"1.8.8\",\"protocol\":47}},\"players\":{{\"max\":100,\"online\":{count},\"sample\":[]}},\"description\":{{\"text\":\"Hello, World!\"}}}}");
                        let packet = ClientBoundPacket::StatusResponse {
                            status,
                        }
                        .serialize();

                        socket.write_all(&packet).await?;
                    },
                    ProxyBoundPacket::StatusPing { payload } => {
                        let mut packet = BytesMut::with_capacity(10);
                        packet.put_u8(9);
                        packet.put_u8(1);
                        packet.put_u64(payload);

                        socket.write_all(&packet).await?;
                    },
                    ProxyBoundPacket::LoginStart { name } => {
                        info!("player {} joining", name);

                        let id = uuid::Uuid::new_v4();
                        let packet = ClientBoundPacket::LoginSuccess {
                            uuid: id.to_string(), // random uuid
                            username: name.clone(),
                        }
                        .serialize();

                        let id = id.as_u128();
                        data.id = Some(id);
                        data.username = Some(name.clone());

                        socket.write_all(&packet).await?;
                        data.state = State::Play;
                        players
                            .send(crate::players::Message::Put(id, tx.clone()))
                            .await?;

                        let packet = makar_protocol::ServerBoundPacket::JoinGameRequest { id, username: name };
                        server.send(packet).await?;
                    },
                    ProxyBoundPacket::ChatMessage { message } => match data {
                        Player { id: Some(id), username: Some(ref username), .. } => {
                            info!("chat: {username}: {message}");
                            let packet = makar_protocol::ServerBoundPacket::ChatMessage { player: id, message };
                            server.send(packet).await?;
                        }
                        _ => {},
                    },
                    ProxyBoundPacket::ClientSettings { locale, view_distance, chat_mode, chat_colors, displayed_skin_parts } => {
                        let packet = makar_protocol::ServerBoundPacket::ClientSettings {
                            player: data.id.unwrap(),
                            locale,
                        };
                        server.send(packet).await?;
                    },
                    ProxyBoundPacket::PlayerPositionAndLook { x, y, z, yaw, pitch, on_ground } => {
                        let packet = ClientBoundPacket::PlayerPositionAndLook { x, y, z, yaw, pitch, flags: 0 }.serialize();
                        socket.write_all(&packet).await?;
                    },
                    ProxyBoundPacket::PlayerPosition { x, y, z, on_ground } => {}
                    ProxyBoundPacket::PlayerIsOnGround { on_ground } => {},
                    ProxyBoundPacket::PluginMessage { channel } => {},
                }
            }
            msg = rx.recv() => {
                match msg {
                    Some(message) => {
                        socket.write_all(&message).await?;
                    }
                    None => {}
                };
            }
        }
    }
}
