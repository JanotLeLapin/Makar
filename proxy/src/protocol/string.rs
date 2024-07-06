use crate::protocol::{Deserialize, Serialize, VarInt};
use bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug, thiserror::Error)]
pub enum StringError {
    #[error("String too long")]
    TooLong,
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
