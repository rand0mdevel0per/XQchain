use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use crate::message::Message;
use crate::error::{NetError, Result};
use std::sync::Arc;

pub struct Peer {
    addr: String,
}

impl Peer {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }

    pub async fn connect(&self) -> Result<PeerConnection> {
        let stream = TcpStream::connect(&self.addr).await?;
        Ok(PeerConnection { stream })
    }
}

pub struct PeerConnection {
    stream: TcpStream,
}

impl PeerConnection {
    pub fn from_stream(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub async fn send(&mut self, msg: &Message) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        let bytes = msg.encode();
        let len = bytes.len() as u32;
        self.stream.write_all(&len.to_le_bytes()).await?;
        self.stream.write_all(&bytes).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Message> {
        use tokio::io::AsyncReadExt;
        let mut len_bytes = [0u8; 4];
        self.stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_le_bytes(len_bytes) as usize;

        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf).await?;

        Message::decode(&buf).map_err(|e| NetError::Serialization(e.to_string()))
    }
}
