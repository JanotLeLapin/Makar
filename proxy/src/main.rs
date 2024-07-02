mod connection;
mod packet;

use std::error::Error;

use tokio::net::TcpListener;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let server = TcpListener::bind("127.0.0.1:25565").await?;

    loop {
        let (socket, _) = server.accept().await?;
        tokio::spawn(async move {
            connection::connection_task(socket).await.unwrap();
        });
    }
}
