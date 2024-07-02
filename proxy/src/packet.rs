use bytes::{Buf, BufMut, Bytes, BytesMut};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VarLenError {
    #[error("VarLen too long")]
    TooLong,
}

#[derive(Debug, Error)]
pub enum StringError {
    #[error("String too long")]
    TooLong,
    #[error("String too short")]
    TooShort,
}

pub trait PacketRead {
    fn get_varint(&mut self) -> Result<i32, VarLenError>;
    fn get_string(&mut self) -> Result<String, StringError>;
}

pub trait PacketWrite {
    fn put_varint(&mut self, value: i32);
    fn put_string(&mut self, string: &str);
}

impl PacketWrite for BytesMut {
    fn put_varint(&mut self, mut value: i32) {
        while value >= 0x80 {
            self.put_u8((value as u8) | 0x80);
            value >>= 7;
        }
        self.put_u8(value as u8);
    }

    fn put_string(&mut self, string: &str) {
        let bytes = string.as_bytes();
        self.put_varint(bytes.len() as i32);
        self.put_slice(bytes);
    }
}

impl PacketRead for Bytes {
    fn get_varint(&mut self) -> Result<i32, VarLenError> {
        let mut res: i32 = 0;
        let mut pos: u8 = 0;
        let mut b: u8;

        loop {
            b = self.get_u8();
            res |= (b as i32 & 0x7F) << pos;
            if (b & 0x80) == 0 {
                break;
            }

            pos += 7;
            if pos >= 32 {
                return Err(VarLenError::TooLong);
            }
        }
        Ok(res)
    }

    fn get_string(&mut self) -> Result<String, StringError> {
        let length = self.get_varint().map_err(|_| StringError::TooLong)? as usize;
        let res = self.split_to(length);
        Ok(String::from_utf8_lossy(&res).to_string())
    }
}

pub fn varint_size(mut value: i32) -> u8 {
    let mut bytes = 1;
    while value >= 0x80 {
        value >>= 7;
        bytes += 1;
    }
    bytes
}
