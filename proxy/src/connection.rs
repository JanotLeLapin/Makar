use std::error::Error;

use tokio::{io::AsyncReadExt, net::TcpStream};

pub async fn connection_task(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
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
    }
}
