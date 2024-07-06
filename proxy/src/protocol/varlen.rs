#[derive(Debug, thiserror::Error)]
pub enum VarLenError {
    #[error("VarLen too long")]
    TooLong,
}

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

        impl crate::protocol::Serialize for $name {
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

        impl crate::protocol::Deserialize for $name {
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
