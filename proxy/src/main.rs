mod connection;
mod players;
mod protocol;
mod server;
mod versions;

use std::error::Error;

use tokio::net::TcpListener;
use tokio::sync::mpsc;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let server = TcpListener::bind("127.0.0.1:25565").await?;

    let (players_tx, players_rx) = mpsc::channel(100);
    tokio::spawn(async move {
        players::players_task(players_rx).await.unwrap();
    });

    let (server_tx, server_rx) = mpsc::channel(100);
    {
        let players_tx = players_tx.clone();
        tokio::spawn(async move {
            server::server_task("127.0.0.1:25566", server_rx, players_tx)
                .await
                .unwrap();
        });
    }

    loop {
        let players_tx = players_tx.clone();
        let server_tx = server_tx.clone();
        let (connection_tx, connection_rx) = mpsc::channel(100);

        let (socket, _) = server.accept().await?;
        tokio::spawn(async move {
            connection::connection_task(
                socket,
                connection_rx,
                connection_tx,
                players_tx,
                server_tx,
            )
            .await
            .unwrap();
        });
    }
}
