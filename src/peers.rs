use surf::Url;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MemberEntry {
    pub peer: String,
}

pub struct Peers {
    pub members: Vec<MemberEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EntryRejectedErr {
    AlreadyPresent(MemberEntry),
    InvalidURL(String),
    Unknown,
}

impl Peers {
    pub fn new() -> Peers {
        Peers { members: vec![] }
    }
    pub fn append(&mut self, entry: MemberEntry) -> Result<MemberEntry, EntryRejectedErr> {
        if Url::parse(&*entry.peer).is_ok() == false {
            return Err(EntryRejectedErr::InvalidURL(entry.peer));
        }
        if self.members.contains(&entry) {
            return Err(EntryRejectedErr::AlreadyPresent(entry));
        }
        self.members.push(entry.clone());
        Ok(entry)
    }
}


impl PartialEq for MemberEntry {
    fn eq(&self, other: &Self) -> bool {
        self.peer == other.peer
    }
}
