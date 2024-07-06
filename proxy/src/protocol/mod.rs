mod chat;
mod primitive;
mod string;
mod varlen;

pub use chat::Chat;
pub use varlen::*;

use bytes::{BufMut, Bytes, BytesMut};

pub trait Serialize {
    fn size(&self) -> i32;
    fn serialize(&self, buf: &mut BytesMut);
}

pub trait Deserialize: Sized {
    type Error;
    fn deserialize(buf: &mut Bytes) -> Result<Self, Self::Error>;
}

impl Serialize for Vec<u8> {
    fn size(&self) -> i32 {
        self.len() as i32
    }

    fn serialize(&self, buf: &mut BytesMut) {
        buf.put_slice(&self);
    }
}

impl Serialize for makar_protocol::Gamemode {
    fn size(&self) -> i32 {
        1
    }

    fn serialize(&self, buf: &mut BytesMut) {
        use makar_protocol::Gamemode::*;
        let b: u8 = match self {
            Survival => 0,
            Creative => 1,
            Adventure => 2,
            Spectator => 3,
        };
        b.serialize(buf);
    }
}

impl Serialize for makar_protocol::Difficulty {
    fn size(&self) -> i32 {
        1
    }

    fn serialize(&self, buf: &mut BytesMut) {
        use makar_protocol::Difficulty::*;
        let b: u8 = match self {
            Peaceful => 0,
            Easy => 1,
            Normal => 2,
            Hard => 3,
        };
        b.serialize(buf);
    }
}

#[derive(Debug)]
pub enum State {
    Handshake,
    Status,
    Login,
    Play,
}

#[macro_export]
macro_rules! define_proxy_bound {
    ($($name:ident, $state:ident, $id:expr => {
        $($field:ident: $type:ty,)*
    }),* $(,)?) => {
        #[derive(Debug)]
        pub enum ProxyBoundPacket {
            $($name {
                $($field: $type,)*
            }),*
        }

        impl ProxyBoundPacket {
            pub fn deserialize(state: &crate::protocol::State, mut packet: bytes::Bytes) -> Result<Self, Box<dyn std::error::Error>> {
                use crate::protocol::Deserialize;

                match (state, crate::protocol::VarInt::deserialize(&mut packet)?.value()) {
                    $((crate::protocol::State::$state, $id) =>
                        Ok(Self::$name {
                            $($field: <$type>::deserialize(&mut packet)?,)*
                        }),
                    )*
                    (state, id) => Err(format!("unknown id {id} for state {state:?}").into())
                }
            }
        }
    };
}

#[macro_export]
macro_rules! define_client_bound {
    ($($name:ident, $id:expr => {
        $($field:ident: $type:ty,)*
    }),* $(,)?) => {
        #[derive(Debug)]
        pub enum ClientBoundPacket {
            $($name {
                $($field: $type,)*
            }),*
        }

        impl ClientBoundPacket {
            pub fn serialize(&self) -> bytes::BytesMut {
                use crate::protocol::{VarInt, Serialize};

                match self {
                    $(Self::$name { $($field,)* } => {
                        let id = VarInt::new($id);
                        let payload_size = 0 $(+ $field.size())*;
                        let size = id.size() + payload_size;
                        let mut packet = bytes::BytesMut::with_capacity(VarInt::new(payload_size).size() as usize + size as usize);

                        VarInt::new(size).serialize(&mut packet);
                        id.serialize(&mut packet);
                        $(let _ = $field.serialize(&mut packet);)*
                        packet
                    }),*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_bound {
    ($name:ident, ClientBound, $($field:ident: $type:ty,)*) => {
        impl $name {
            pub fn payload_size(&self) -> i32 {
                use crate::protocol::Serialize;

                0 $(+ self.$field.size())*
            }

            pub fn serialize(&self) -> bytes::BytesMut {
                use crate::protocol::{VarInt, Serialize};

                let payload_size = self.payload_size();
                let size = VarInt::new($name::packet_id()).size() + payload_size;
                let mut packet = bytes::BytesMut::with_capacity(VarInt::new(payload_size).size() as usize + size as usize);

                VarInt::new(size).serialize(&mut packet);
                VarInt::new($name::packet_id()).serialize(&mut packet);
                $(let _ = &self.$field.serialize(&mut packet);)*
                packet
            }
        }
    };
    ($name:ident, ServerBound, $($field:ident: $type:ty,)*) => {
        impl $name {
            pub fn deserialize(mut packet: bytes::Bytes) -> Result<Self, Box<dyn std::error::Error>> {
                use crate::protocol::Deserialize;

                Ok(Self {
                    $($field: <$type>::deserialize(&mut packet)?,)*
                })
            }
        }
    }
}
