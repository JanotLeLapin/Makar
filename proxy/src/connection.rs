use std::error::Error;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
};

use crate::protocol::{Deserialize, VarInt};
use crate::versions::v1_8_8::*;

const STATUS: &'static str = "{\"version\":{\"name\":\"1.8.8\",\"protocol\":47},\"players\":{\"max\":100,\"online\":0,\"sample\":[]},\"description\":{\"text\":\"Hello, World!\"}}";

pub enum State {
    Handshake,
    Status,
    Login,
    Play,
}

pub async fn connection_task(
    mut socket: TcpStream,
    mut rx: mpsc::Receiver<Vec<u8>>,
    tx: mpsc::Sender<Vec<u8>>,
    players: mpsc::Sender<crate::players::Message>,
    server: mpsc::Sender<makar_protocol::ServerBoundPacket>,
) -> Result<(), Box<dyn Error>> {
    let mut protocol: u16 = 0;
    let mut state = State::Handshake;
    loop {
        tokio::select! {
            b = socket.read_u8() => {
                let size = {
                    let mut res: i32 = 0;
                    let mut pos: u8 = 0;
                    let mut b = b?;

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

                println!("{:?}", buf);

                let mut bytes = Bytes::from(buf);
                let id = VarInt::deserialize(&mut bytes)?.value();

                match state {
                    State::Handshake => {
                        let packet = Handshake::deserialize(bytes)?;
                        println!("{packet:?}");
                        state = match packet.next_state {
                            1 => State::Status,
                            2 => State::Login,
                            v => panic!("unknown state {v}"),
                        };
                    }
                    State::Status => match id {
                        0x00 => {
                            let packet = StatusResponse {
                                status: STATUS.to_string(),
                            }
                            .serialize();

                            socket.write_all(&packet).await?;
                        }
                        0x01 => {
                            let mut packet = BytesMut::with_capacity(10);
                            packet.put_u8(9);
                            packet.put_u8(1);
                            packet.put_u64(bytes.get_u64());

                            socket.write_all(&packet).await?;
                        }
                        _ => unimplemented!("id {id} for status"),
                    },
                    State::Login => match id {
                        0x00 => {
                            let packet = LoginStart::deserialize(bytes)?;
                            let id = uuid::Uuid::new_v4();
                            let packet = LoginSuccess {
                                uuid: id.to_string(), // random uuid
                                username: packet.name,
                            }
                            .serialize();
                            socket.write_all(&packet).await?;
                            state = State::Play;
                            players
                                .send(crate::players::Message::Put(id.as_u128(), tx.clone()))
                                .await?;

                            let packet = makar_protocol::ServerBoundPacket::JoinGameRequest(id.as_u128());
                            println!("proxy -> server ({packet:?})");
                            server.send(packet).await?;
                        }
                        _ => unimplemented!("id {id} for login"),
                    },
                    State::Play => match id {
                        _ => unimplemented!("id {id} for play"),
                    },
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
