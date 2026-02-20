use serde::{Serialize, Deserialize};
use xcqa_chain_core::{Block, Transaction};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

#[derive(Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
pub enum Message {
    Handshake { version: u32, peer_id: [u8; 32] },
    Block(Block),
    Transaction(Transaction),
    GetBlocks { start_height: u64, count: u32 },
    Ping,
    Pong,
}

impl Message {
    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self).unwrap().to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, rkyv::rancor::Error> {
        rkyv::from_bytes(bytes)
    }
}
