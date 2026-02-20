use tokio::net::TcpListener;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{Peer, PeerConnection, Message, Result};

pub struct NetworkManager {
    peers: Arc<RwLock<Vec<PeerConnection>>>,
    port: u16,
}

impl NetworkManager {
    pub fn new(port: u16) -> Self {
        Self {
            peers: Arc::new(RwLock::new(Vec::new())),
            port,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;

        loop {
            let (stream, _) = listener.accept().await?;
            let conn = PeerConnection::from_stream(stream);
            self.peers.write().await.push(conn);
        }
    }

    pub async fn connect_peer(&self, addr: String) -> Result<()> {
        let peer = Peer::new(addr);
        let conn = peer.connect().await?;
        self.peers.write().await.push(conn);
        Ok(())
    }

    pub async fn broadcast(&self, msg: Message) -> Result<()> {
        let mut peers = self.peers.write().await;
        for peer in peers.iter_mut() {
            let _ = peer.send(&msg).await;
        }
        Ok(())
    }
}
