use serde::{Deserialize, Serialize};
use serde_json::{Value};
use std::collections::HashMap;
use sha2::{Digest, Sha256};
extern crate base64;
extern crate hex;
use std::time::{SystemTime, UNIX_EPOCH};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: u128,
    pub data: HashMap<String, Value>,
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index &&
        self.previous_hash == other.previous_hash &&
        self.timestamp == other.timestamp &&
        self.data == other.data
    }
}

pub fn message_as_json(message: &str) -> HashMap<String, Value> {
    let data_str = format!(r#"
    {{
        "message": "{message}"
    }}"#, message=message);
    serde_json::from_str(&data_str).unwrap()
}

fn calculate_hash(index: u64, timestamp: u128, previous_hash: &str, data: &str) -> Vec<u8> {
    let data = serde_json::json!({
        "index": index,
        "previous_hash": previous_hash,
        "data": data,
        "timestamp": timestamp.to_string()
    });
    let mut hasher = Sha256::new();
    hasher.update(data.to_string().as_bytes());
    hasher.finalize().as_slice().to_owned()
}

fn as_hex(bytes: Vec<u8>) -> String {
    hex::encode(&bytes)
}

pub fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

impl Block {
    pub fn hash(&self) -> String {
        let serialized_data = serde_json::to_string(&self.data).unwrap();
        as_hex(calculate_hash(self.index, self.timestamp, &self.previous_hash, &serialized_data))
    }

    pub fn generate_next(&self, message:String) -> Block {
        Block{
            index: self.index + 1,
            previous_hash: self.hash(),
            timestamp: get_epoch_ms(),
            data: message_as_json(&message)
        }
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_attribute() {
        let genesis = Block{
            index: 0,
            previous_hash: String::from(""),
            timestamp: 0,
            data: message_as_json("not important")
        };
        assert_eq!(genesis.index, 0)
    }

    #[test]
    fn test_data_attribute() {
        let genesis = Block{
            index: 0,
            previous_hash: String::from(""),
            timestamp: 0,
            data: message_as_json("This data has to match")
        };
        let expected_data = message_as_json("This data has to match");
        assert_eq!(genesis.data, expected_data);
    }

    #[test]
    fn test_timestamp_attribute() {
        let now = get_epoch_ms();
        let genesis = Block{
            index: 0,
            previous_hash: String::from(""),
            timestamp: now,
            data: message_as_json("This timestamp has to match")
        };
        assert_eq!(genesis.timestamp, now)
    }

    #[test]
    fn test_hash_method() {
        let genesis = Block{
            index: 0,
            previous_hash: String::from(""),
            timestamp: 0,
            data: message_as_json("not important")
        };
        let expected_hash = "ffd175853d16c15f4a97051c906bdb60fafd2e67a6ed6e179a66cdc91876156f";
        assert_eq!(expected_hash, genesis.hash())
    }

    #[test]
    fn test_is_equal() {
        let one = Block{
            index: 0,
            previous_hash: String::from(""),
            timestamp: 0,
            data: message_as_json("not important")
        };
        let another = Block{
            index: 0,
            previous_hash: String::from(""),
            timestamp: 0,
            data: message_as_json("not important")
        };
        assert_eq!(one == another, true)
    }

    #[test]
    fn test_is_not_equal_by_index() {
        let one = Block{
            index: 0,
            previous_hash: String::from(""),
            timestamp: 0,
            data: message_as_json("Not important")
        };
        let another = Block{
            index: one.index + 1,
            previous_hash: String::from(""),
            timestamp: 0,
            data: message_as_json("Not important")
        };
        assert_eq!(one != another, true)
    }

    #[test]
    fn test_is_not_equal_by_previous_hash() {
        let one = Block{
            index: 0,
            previous_hash: String::from("000000000000000"),
            timestamp: 0,
            data: message_as_json("Not important")
        };
        let another = Block{
            index: 0,
            previous_hash: String::from("fffffffffffffff"),
            timestamp: 0,
            data: message_as_json("Not important")
        };
        assert_eq!(one != another, true)
    }

    #[test]
    fn test_is_not_equal_by_timestamp() {
        let one = Block{
            index: 0,
            previous_hash: String::from(""),
            timestamp: 0,
            data: message_as_json("Not important")
        };
        let another = Block{
            index: 0,
            previous_hash: String::from(""),
            timestamp: 123456789,
            data: message_as_json("Not important")
        };
        assert_eq!(one != another, true)
    }

    #[test]
    fn test_is_not_equal_by_content() {
        let one = Block{
            index: 0,
            previous_hash: String::from(""),
            timestamp: 0,
            data: message_as_json("Not important")
        };
        let another = Block{
            index: 0,
            previous_hash: String::from(""),
            timestamp: 0,
            data: message_as_json("This is a different data")
        };
        assert_eq!(one != another, true)
    }

    #[test]
    fn test_genesis_valid_next() {
        let genesis = Block{
            index: 0,
            previous_hash: String::from(""),
            timestamp: 0,
            data: message_as_json("Not important")
        };
        let next_block = genesis.generate_next(String::from("New data"));
        assert_eq!(next_block.previous_hash == genesis.hash(), true)
    }
}
