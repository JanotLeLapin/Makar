use bytes::{BufMut, Bytes, BytesMut};

pub trait Serialize {
    fn size(&self) -> i32;
    fn serialize(&self, buf: &mut BytesMut);
}

pub trait Deserialize: Sized {
    type Error;
    fn deserialize(buf: &mut Bytes) -> Result<Self, Self::Error>;
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Chat {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlined: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obfuscated: Option<bool>,
}

impl From<makar_protocol::Chat> for Chat {
    fn from(value: makar_protocol::Chat) -> Self {
        let makar_protocol::Chat {
            text,
            color,
            bold,
            italic,
            underlined,
            strikethrough,
            obfuscated,
        } = value;
        Self {
            text,
            color,
            bold,
            italic,
            underlined,
            strikethrough,
            obfuscated,
        }
    }
}

impl Serialize for String {
    fn size(&self) -> i32 {
        let len = self.as_bytes().len() as i32;
        len + VarInt::new(len).size()
    }

    fn serialize(&self, buf: &mut BytesMut) {
        let bytes = self.as_bytes();
        VarInt::new(bytes.len() as i32).serialize(buf);
        buf.put_slice(bytes);
    }
}

impl Deserialize for String {
    type Error = StringError;
    fn deserialize(buf: &mut Bytes) -> Result<Self, Self::Error> {
        let length = VarInt::deserialize(buf)
            .map_err(|_| StringError::TooLong)?
            .value() as usize;
        let res = buf.split_to(length);
        Ok(String::from_utf8_lossy(&res).to_string())
    }
}

impl Serialize for Vec<u8> {
    fn size(&self) -> i32 {
        self.len() as i32
    }

    fn serialize(&self, buf: &mut BytesMut) {
        buf.put_slice(&self);
    }
}

macro_rules! primitive {
    ($type:ty, $put:ident, $get:ident, $size:expr) => {
        impl Serialize for $type {
            fn size(&self) -> i32 {
                $size
            }

            fn serialize(&self, buf: &mut bytes::BytesMut) {
                use bytes::BufMut;

                buf.$put(*self);
            }
        }

        impl Deserialize for $type {
            type Error = std::convert::Infallible;
            fn deserialize(buf: &mut bytes::Bytes) -> Result<Self, Self::Error> {
                use bytes::Buf;

                Ok(buf.$get())
            }
        }
    };
}

primitive!(u8, put_u8, get_u8, 1);
primitive!(i8, put_i8, get_i8, 1);
primitive!(u16, put_u16, get_u16, 2);
primitive!(i16, put_i16, get_i16, 2);
primitive!(u32, put_u32, get_u32, 4);
primitive!(i32, put_i32, get_i32, 4);
primitive!(u64, put_u64, get_u64, 8);
primitive!(i64, put_i64, get_i64, 8);
primitive!(f32, put_f32, get_f32, 4);
primitive!(f64, put_f64, get_f64, 8);

macro_rules! varlen {
    ($name:ident, $type:ty) => {
        #[derive(Debug)]
        pub struct $name($type);

        impl $name {
            pub fn new(value: $type) -> Self {
                Self(value)
            }

            pub fn value(&self) -> $type {
                self.0
            }
        }

        impl Serialize for $name {
            fn size(&self) -> i32 {
                let mut value = self.value();
                let mut size = 1;
                while value >= 0x80 {
                    value >>= 7;
                    size += 1;
                }
                size
            }

            fn serialize(&self, buf: &mut bytes::BytesMut) {
                use bytes::BufMut;

                let mut value = self.value();
                while (value & 0x80) == 0x80 {
                    buf.put_u8((value as u8) | 0x80);
                    value >>= 7;
                }
                buf.put_u8(value as u8);
            }
        }

        impl Deserialize for $name {
            type Error = VarLenError;
            fn deserialize(buf: &mut bytes::Bytes) -> Result<Self, Self::Error> {
                use bytes::Buf;

                let mut res: $type = 0;
                let mut pos: u8 = 0;
                let mut b: u8;

                loop {
                    b = buf.get_u8();
                    res |= (b as $type & 0x7F) << pos;
                    if (b & 0x80) == 0 {
                        break;
                    }

                    pos += 7;
                    if pos >= 32 {
                        return Err(VarLenError::TooLong);
                    }
                }
                Ok(Self::new(res))
            }
        }
    };
}

varlen!(VarInt, i32);
varlen!(VarLong, i64);

impl Serialize for Chat {
    fn size(&self) -> i32 {
        serde_json::to_string(self).unwrap().size()
    }

    fn serialize(&self, buf: &mut BytesMut) {
        serde_json::to_string(self).unwrap().serialize(buf);
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

#[derive(Debug, thiserror::Error)]
pub enum VarLenError {
    #[error("VarLen too long")]
    TooLong,
}

#[derive(Debug, thiserror::Error)]
pub enum StringError {
    #[error("String too long")]
    TooLong,
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
