use crate::blockchain::InvalidBlockErr;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct APIErrorAndReason {
    pub error: String,
    pub reason: String,
}

const HASH_NOT_MATCHING_LABEL: &str = "Previous hash not matching";
const INDEX_NOT_CORRELATIVE_LABEL: &str = "New block index is not correlative";
const TIMESTAMP_NOT_LATER_LABEL: &str = "New block timestamp must be later to previous";
lazy_static! {
    pub static ref HASH_NOT_MATCHING_DESC_REGEX: Regex =
        Regex::new(r"previous hash is ([a-f0-9]{32}) but ([a-f0-9]{32}) was provided").unwrap();
    pub static ref NOT_CORRELATIVE_DESC_REGEX: Regex =
        Regex::new(r"expected index (\d+) but received (\d+) which is not inmediate next").unwrap();
    pub static ref NOT_POSTERIOR_DESC_REGEX: Regex =
        Regex::new(r"Given timestamp (\d+) is not later to (\d+)").unwrap();
}

fn params_for_hash_not_matching(reason: String) -> (String, String) {
    let caps = HASH_NOT_MATCHING_DESC_REGEX.captures(&*reason).unwrap();
    (
        String::from(caps.get(2).map_or("", |m| m.as_str())),
        String::from(caps.get(1).map_or("", |m| m.as_str())),
    )
}

fn params_for_not_correlative(reason: String) -> (u64, u64) {
    let caps = NOT_CORRELATIVE_DESC_REGEX.captures(&*reason).unwrap();
    (
        caps.get(2)
            .map_or(0, |m| m.as_str().parse::<u64>().unwrap()),
        caps.get(1)
            .map_or(0, |m| m.as_str().parse::<u64>().unwrap()),
    )
}

fn params_for_not_posterior(reason: String) -> (u128, u128) {
    let caps = NOT_POSTERIOR_DESC_REGEX.captures(&*reason).unwrap();
    (
        caps.get(1)
            .map_or(0, |m| m.as_str().parse::<u128>().unwrap()),
        caps.get(2)
            .map_or(0, |m| m.as_str().parse::<u128>().unwrap()),
    )
}

impl InvalidBlockErr {
    pub fn as_api_error(&self) -> APIErrorAndReason {
        match self {
            InvalidBlockErr::HashNotMatching(given, expected) => {
                let reason = format!("previous hash is {} but {} was provided", expected, given);
                APIErrorAndReason {
                    error: String::from(HASH_NOT_MATCHING_LABEL),
                    reason: String::from(reason),
                }
            }
            InvalidBlockErr::NotCorrelated(given, expected) => {
                let reason = format!(
                    "expected index {} but received {} which is not inmediate next",
                    expected, given
                );
                APIErrorAndReason {
                    error: String::from(INDEX_NOT_CORRELATIVE_LABEL),
                    reason: String::from(reason),
                }
            }
            InvalidBlockErr::NotPosterior(given, expected) => {
                let reason = format!("Given timestamp {} is not later to {}", given, expected);
                APIErrorAndReason {
                    error: String::from(TIMESTAMP_NOT_LATER_LABEL),
                    reason: String::from(reason),
                }
            }
            _ => APIErrorAndReason {
                error: String::from("Unknown error"),
                reason: String::from("reason"),
            },
        }
    }
}

impl APIErrorAndReason {
    pub fn as_native_error(self) -> InvalidBlockErr {
        match &*self.error {
            HASH_NOT_MATCHING_LABEL => {
                let (expected, given) = params_for_hash_not_matching(self.reason);
                InvalidBlockErr::HashNotMatching(expected, given)
            }
            INDEX_NOT_CORRELATIVE_LABEL => {
                let (expected, given) = params_for_not_correlative(self.reason);
                InvalidBlockErr::NotCorrelated(expected, given)
            }
            TIMESTAMP_NOT_LATER_LABEL => {
                let (expected, given) = params_for_not_posterior(self.reason);
                InvalidBlockErr::NotPosterior(expected, given)
            }
            _ => InvalidBlockErr::Unkown,
        }
    }
}
