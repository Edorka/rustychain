use crate::blockchain::block::Block;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct BlockList {
    pub items: Vec<Block>,
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
