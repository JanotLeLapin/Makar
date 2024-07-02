use std::error::Error;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::packet::{PacketRead, PacketWrite};

const STATUS: &'static str = "{\"version\":{\"name\":\"1.8.8\",\"protocol\":47},\"players\":{\"max\":100,\"online\":0,\"sample\":[]},\"description\":{\"text\":\"Hello, World!\"}}";

pub enum State {
    HANDSHAKE,
    STATUS,
    LOGIN,
    PLAY,
}

pub async fn connection_task(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut protocol: u16 = 0;
    let mut state = State::HANDSHAKE;
    loop {
        let size = {
            let mut res: i32 = 0;
            let mut pos: u8 = 0;
            let mut b: u8;

            loop {
                b = socket.read_u8().await?;
                res |= (b as i32 & 0x7F) << pos;
                if (b & 0x80) == 0 {
                    break Some(res);
                }

                pos += 7;
                if pos >= 32 {
                    break None;
                }
            }
        }
        .expect("Packet size too big") as usize;

        let mut buf = vec![0u8; size as usize];
        socket.read_exact(&mut buf).await?;

        println!("{:?}", buf);

        let mut bytes = Bytes::from(buf);
        let id = bytes.get_varint()?;

        match state {
            State::HANDSHAKE => {
                protocol = bytes.get_varint()? as u16;
                let _address = bytes.get_string()?;
                let _port = bytes.get_u16();
                state = match bytes.get_u8() {
                    1 => State::STATUS,
                    2 => State::LOGIN,
                    v => panic!("unknown state {v}"),
                };
            }
            State::STATUS => match id {
                0x00 => {
                    let status_len = STATUS.as_bytes().len() as i32;
                    let status_size_len = crate::packet::varint_size(status_len) as i32;
                    let packet_len = 1 + status_size_len + status_len;
                    let packet_size_len = crate::packet::varint_size(packet_len) as i32;

                    let mut packet =
                        BytesMut::with_capacity(packet_len as usize + packet_size_len as usize);
                    packet.put_varint(packet_len);
                    packet.put_u8(0x00);
                    packet.put_string(STATUS);

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
            State::LOGIN => match id {
                _ => unimplemented!("id {id} for login"),
            },
            State::PLAY => match id {
                _ => unimplemented!("id {id} for play"),
            },
        }
    }
}
