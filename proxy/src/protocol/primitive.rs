macro_rules! primitive {
    ($type:ty, $put:ident, $get:ident, $size:expr) => {
        impl crate::protocol::Serialize for $type {
            fn size(&self) -> i32 {
                $size
            }

            fn serialize(&self, buf: &mut bytes::BytesMut) {
                use bytes::BufMut;

                buf.$put(*self);
            }
        }

        impl crate::protocol::Deserialize for $type {
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
