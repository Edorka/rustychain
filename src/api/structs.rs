use crate::blockchain::block::Block;
use crate::blockchain::{Chain, InvalidBlockErr};
use crate::peers::{Peers, MemberEntry, EntryRejectedErr};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Serialize, Debug)]
pub struct List<T> {
    pub items: Vec<T>,
}

pub type BlockList = List<Block>;
pub type PeerList = List<MemberEntry>;

#[derive(Deserialize)]
#[serde(default)]
pub struct Limits {
    pub from_index: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self { from_index: 0 }
    }
}
impl Limits {
    pub fn as_query(&self) -> String {
        format!("from_index={}", &self.from_index)
    }
}

#[derive(Clone)]
pub struct State {
    pub chain: Arc<Mutex<Chain>>,
    pub peers: Arc<Mutex<Peers>>,
}

impl State {
    pub fn new(genesis_data: String) -> Self {
        Self {
            chain: Arc::new(Mutex::new(Chain::new(genesis_data))),
            peers: Arc::new(Mutex::new(Peers::new())),
        }
    }
    pub fn append_block(&self, block: Block) -> Result<Block, InvalidBlockErr> {
        let mut chain = self.chain.lock().unwrap();
        chain.append(block)
    }
    pub fn add_peer(&self, entry: MemberEntry) -> Result<MemberEntry, EntryRejectedErr> {
        let mut peers = self.peers.lock().unwrap();
        peers.append(entry)
    }
}
