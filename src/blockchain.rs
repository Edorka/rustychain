pub mod block;
use block::{Block, get_epoch_ms, message_as_json};


#[derive(Debug, PartialEq, Clone)]
pub enum InvalidBlockErr {
    NotCorrelated(u64, u64),
    NotPosterior(u128, u128),
    HashNotMatching(String, String),
    GenesisBlockNotFound
}

pub struct Chain {
    pub blocks: Vec<Block>,
}


impl Chain {
    pub fn new(initial_message: String) -> Chain {
        let data = message_as_json(&initial_message);
        let genesis_block = Block{
            index: 0,
            data: data.clone(),
            previous_hash: String::from(""),
            timestamp: get_epoch_ms()
        };
        Chain{ blocks: vec![genesis_block] }
    }
    pub fn append(&mut self, block: Block) -> Result<Block, InvalidBlockErr> {
        if self.blocks.len() == 0 {
            return Err(InvalidBlockErr::GenesisBlockNotFound);
        }
        let last = self.blocks.last().unwrap();
        if block.index != (last.index + 1) {
            return Err(InvalidBlockErr::NotCorrelated(block.index, last.index))
        }
        if block.timestamp < last.timestamp {
            return Err(InvalidBlockErr::NotPosterior(block.timestamp, last.timestamp))
        }
        if block.previous_hash != last.hash() {
            return Err(InvalidBlockErr::HashNotMatching(block.previous_hash, last.hash()))
        }
        self.blocks.push(block.clone());
        Ok(block)
    }
    pub fn get_last_block(&self) -> Option<&Block> {
        self.blocks.last()
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;

    fn arrange_a_chain() -> Chain {
        Chain::new(String::from("Genesis block"))
    }

    #[test]
    fn test_genesis_block_not_found() {
        let mut chain = Chain{ blocks: vec![] };
        let next_block = Block{
            index: 1,
            timestamp: 0,
            data: message_as_json("another block"),
            previous_hash: String::from("c4f3c4f3c4f3"),
        };
        let obtained_error = chain.append(next_block).unwrap_err();
        matches!(obtained_error, InvalidBlockErr::GenesisBlockNotFound);
    }

    #[test]
    fn test_invalid_index() {
        let mut chain = arrange_a_chain();
        let next_block = Block{
            index: 5,
            timestamp: chain.blocks[0].timestamp + 100,
            data: message_as_json("another block"),
            previous_hash: chain.blocks[0].hash()
        };
        let obtained_error = chain.append(next_block).unwrap_err();
        let expected_error = InvalidBlockErr::NotCorrelated(0, 5);
        assert!(matches!(obtained_error, expected_error));
    }

    #[test]
    fn test_invalid_timestamp() {
        let mut chain = arrange_a_chain();
        let genesis_timestamp = chain.blocks[0].timestamp;
        let invalid_timestamp = genesis_timestamp - 5;
        let next_block = Block{
            index: 1,
            timestamp: invalid_timestamp,
            data: message_as_json("another block"),
            previous_hash: chain.blocks[0].hash()
        };
        let expected_error = InvalidBlockErr::NotPosterior(genesis_timestamp, invalid_timestamp);
        assert!(matches!(
            chain.append(next_block),
            Err(expected_error)
        ));
    }

    #[test]
    fn test_invalid_hash() {
        let mut chain = arrange_a_chain();
        let invalid_hash = String::from("cafecafecafe");
        let next_block = Block{
            index: 1,
            timestamp: chain.blocks[0].timestamp + 5,
            data: message_as_json("another block"),
            previous_hash: invalid_hash.clone()
        };
        let expected_hash = chain.blocks[0].hash();
        let expected_error = InvalidBlockErr::HashNotMatching(expected_hash, invalid_hash);
        let obtained_error = chain.append(next_block).unwrap_err();
        matches!(obtained_error, expected_error);
    }

    #[test]
    fn test_adding_blocks() {
        let mut chain = arrange_a_chain();
        let next_block = Block{
            index: 1,
            timestamp: chain.blocks[0].timestamp + 100,
            data: message_as_json("another block"),
            previous_hash: chain.blocks[0].hash()
        };
        let expected_block = next_block.clone();
        let added_block = chain.append(next_block);
        matches!(added_block, expected_block);
        assert!(chain.blocks.contains(&expected_block))
    }

}
