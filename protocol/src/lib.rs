use bytes::{BufMut, BytesMut};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerBoundPacket {
    JoinGameRequest(u128),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProxyBoundPacket {
    JoinGame {
        player: u128,
        entity_id: i32,
        gamemode: u8,
        dimension: i8,
        difficulty: u8,
        max_players: u8,
        level_type: String,
        reduced_debug_info: bool,
    },
    ChatMessage {
        player: u128,
        json: String,
        position: u8,
    },
}

macro_rules! packet_impl {
    ($name:ident) => {
        impl $name {
            pub fn deserialize(buf: &[u8]) -> postcard::Result<Self> {
                postcard::from_bytes(buf)
            }

            pub fn serialize(&self) -> postcard::Result<BytesMut> {
                let buf = postcard::to_allocvec(self)?;
                let len = buf.len();

                let mut res = BytesMut::with_capacity(len + 4);
                res.put_u32(len as u32);
                res.put_slice(&buf);

                Ok(res)
            }
        }
    };
}

packet_impl!(ServerBoundPacket);
packet_impl!(ProxyBoundPacket);
