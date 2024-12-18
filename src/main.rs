use anyhow::Result;
use simple_redis::{backend::Backend, network};
use tokio::net::TcpListener;
use tracing::{info, warn};
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let addr = "0.0.0.0:6378";
    info!("Simple-Redis-Server is listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    let backend = Backend::new();
    loop {
        let (socket, raddr) = listener.accept().await?;
        info!("Accept connection from {}", raddr);
        let backend_cloned = backend.clone();
        tokio::spawn(async move {
            if let Err(e) = network::stream_handler(socket, backend_cloned).await {
                warn!("Handle error for {}: {:?}", raddr, e);
            }
        });
    }
}
