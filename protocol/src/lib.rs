use bytes::{BufMut, BytesMut};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Gamemode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chat {
    pub text: String,
    pub color: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerBoundPacket {
    JoinGameRequest { id: u128, username: String },
    ClientSettings { player: u128, locale: String },
    ChatMessage { player: u128, message: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProxyBoundPacket {
    JoinGame {
        player: u128,
        entity_id: i32,
        gamemode: Gamemode,
        dimension: i8,
        difficulty: Difficulty,
        max_players: u8,
        level_type: String,
        reduced_debug_info: bool,
    },
    ChatMessage {
        player: u128,
        json: Chat,
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
