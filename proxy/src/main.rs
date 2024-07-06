mod connection;
mod players;
mod protocol;
mod server;
mod versions;

use std::error::Error;

use tokio::net::TcpListener;
use tokio::sync::mpsc;

use log::{error, info, warn};

#[derive(Clone)]
pub struct ProxyContext {
    pub players_tx: mpsc::Sender<players::Message>,
    pub server_tx: mpsc::Sender<makar_protocol::ServerBoundPacket>,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let (players_tx, players_rx) = mpsc::channel(100);
    let (server_tx, server_rx) = mpsc::channel(100);
    let ctx = ProxyContext {
        players_tx,
        server_tx,
    };

    tokio::spawn(async move {
        match players::players_task(players_rx).await {
            Ok(_) => {}
            Err(e) => {
                error!("players task ended unexpectingly: {e}");
            }
        };
    });

    {
        let ctx = ctx.clone();
        tokio::spawn(async move {
            match server::server_task("127.0.0.1:25566", server_rx, ctx).await {
                Ok(_) => {}
                Err(e) => {
                    error!("server task ended unexpectingly: {e}");
                }
            };
        });
    }

    let server = TcpListener::bind("127.0.0.1:25565").await?;
    info!("accepting connections on port 25565");

    loop {
        let ctx = ctx.clone();
        let (connection_tx, connection_rx) = mpsc::channel(100);

        let (socket, addr) = match server.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                warn!("couldn't accept new connection: {e}");
                continue;
            }
        };
        tokio::spawn(async move {
            match connection::connection_task(socket, connection_rx, connection_tx, ctx).await {
                Ok(_) => {}
                Err(e) => {
                    warn!("{addr} connection ended unexpectingly: {e}");
                }
            };
        });
    }
}
