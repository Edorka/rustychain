use crate::blockchain::block::Block;
use crate::blockchain::{Chain, InvalidBlockErr};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use surf::Url;

#[derive(Deserialize, Serialize, Debug)]
pub struct List<T> {
    pub items: Vec<T>,
}

pub type BlockList = List<Block>;
pub type PeerList = List<MemberEntry>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MemberEntry {
    pub peer: String,
}

pub struct Peers {
    pub members: Vec<MemberEntry>,
}

impl PartialEq for MemberEntry {
    fn eq(&self, other: &Self) -> bool {
        self.peer == other.peer
    }
}

impl Peers {
    pub fn new() -> Peers {
        Peers { members: vec![] }
    }
    pub fn append(&mut self, entry: MemberEntry) -> Result<MemberEntry, EntryRejectedErr> {
        if Url::parse(&*entry.peer).is_ok() == false {
            return Err(EntryRejectedErr::Invalid(entry.peer));
        }
        if self.members.contains(&entry) {
            return Err(EntryRejectedErr::AlreadyPresent(entry));
        }
        self.members.push(entry.clone());
        Ok(entry)
    }
}

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

pub enum EntryRejectedErr {
    AlreadyPresent(MemberEntry),
    Invalid(String),
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
