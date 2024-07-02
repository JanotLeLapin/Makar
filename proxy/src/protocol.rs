use bytes::{BufMut, Bytes, BytesMut};

pub trait Serialize {
    fn size(&self) -> i32;
    fn serialize(&self, buf: &mut BytesMut);
}

pub trait Deserialize: Sized {
    type Error;
    fn deserialize(buf: &mut Bytes) -> Result<Self, Self::Error>;
}

impl Serialize for String {
    fn size(&self) -> i32 {
        self.as_bytes().len() as i32
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

macro_rules! primitive {
    ($type:ty, $put:ident, $get:ident) => {
        impl Serialize for $type {
            fn size(&self) -> i32 {
                (<$type>::BITS / 8) as i32
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

primitive!(u8, put_u8, get_u8);
primitive!(i8, put_i8, get_i8);
primitive!(u16, put_u16, get_u16);
primitive!(i16, put_i16, get_i16);
primitive!(u32, put_u32, get_u32);
primitive!(i32, put_i32, get_i32);
primitive!(u64, put_u64, get_u64);
primitive!(i64, put_i64, get_i64);

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
                while value >= 0x80 {
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

#[macro_export]
macro_rules! define_protocol {
    ($($name:ident, $id:expr, $bound:ident => {
        $($field:ident: $type:ty,)*
    }),* $(,)?) => {
        $(
            #[derive(Debug)]
            pub struct $name {
                $(pub $field: $type,)*
            }

            impl $name {
                pub fn packet_id() -> i32 {
                    $id
                }
            }

            crate::impl_bound!($name, $bound, $($field: $type,)*);
        )*
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
                let size = VarInt::new(payload_size).size() + VarInt::new($name::packet_id()).size() + payload_size;
                let mut packet = bytes::BytesMut::with_capacity(size as usize);

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
