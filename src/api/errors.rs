use crate::api::structs::{EntryRejectedErr, MemberEntry};
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

const ENTRY_ALREADY_PRESENT_LABEL: &str = "Entry is already on list";
const ENTRY_URL_INVALID_LABEL: &str = "Invalid entry URL";

lazy_static! {
    pub static ref HASH_NOT_MATCHING_DESC_REGEX: Regex =
        Regex::new(r"previous hash is ([a-f0-9]{32}) but ([a-f0-9]{32}) was provided").unwrap();
    pub static ref NOT_CORRELATIVE_DESC_REGEX: Regex =
        Regex::new(r"expected index (\d+) but received (\d+) which is not inmediate next").unwrap();
    pub static ref NOT_POSTERIOR_DESC_REGEX: Regex =
        Regex::new(r"Given timestamp (\d+) is not later to (\d+)").unwrap();
    pub static ref ENTRY_ALREADY_PRESENT_DESC_REGEX: Regex =
        Regex::new(r"Entry is already a member: (.*)$").unwrap();
    pub static ref ENTRY_INVALID_URL_DESC_REGEX: Regex =
        Regex::new(r"Entry URL is invalid: (.*)$").unwrap();
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

fn param_for_entry_invalid_url(reason: String) -> String {
    let caps = ENTRY_INVALID_URL_DESC_REGEX.captures(&*reason).unwrap();
    let input: &str = caps.get(1).unwrap().as_str();
    String::from(input)
}

fn param_for_entry_already_present(reason: String) -> MemberEntry {
    let caps = ENTRY_INVALID_URL_DESC_REGEX.captures(&*reason).unwrap();
    let input: &str = caps.get(1).unwrap().as_str();
    MemberEntry {
        peer: String::from(input),
    }
}

impl From<InvalidBlockErr> for APIErrorAndReason {
    fn from(native_error: InvalidBlockErr) -> Self {
        match native_error {
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

impl From<APIErrorAndReason> for InvalidBlockErr {
    fn from(api_error: APIErrorAndReason) -> Self {
        match &*api_error.error {
            HASH_NOT_MATCHING_LABEL => {
                let (expected, given) = params_for_hash_not_matching(api_error.reason);
                InvalidBlockErr::HashNotMatching(expected, given)
            }
            INDEX_NOT_CORRELATIVE_LABEL => {
                let (expected, given) = params_for_not_correlative(api_error.reason);
                InvalidBlockErr::NotCorrelated(expected, given)
            }
            TIMESTAMP_NOT_LATER_LABEL => {
                let (expected, given) = params_for_not_posterior(api_error.reason);
                InvalidBlockErr::NotPosterior(expected, given)
            }
            _ => InvalidBlockErr::Unkown,
        }
    }
}

impl From<APIErrorAndReason> for EntryRejectedErr {
    fn from(api_error: APIErrorAndReason) -> Self {
        match &*api_error.error {
            ENTRY_URL_INVALID_LABEL => {
                let expected = param_for_entry_invalid_url(api_error.reason);
                EntryRejectedErr::InvalidURL(expected)
            }
            ENTRY_ALREADY_PRESENT_LABEL => {
                let expected = param_for_entry_already_present(api_error.reason);
                EntryRejectedErr::AlreadyPresent(expected)
            }
            _ => EntryRejectedErr::Unknown,
        }
    }
}

impl From<EntryRejectedErr> for APIErrorAndReason {
    fn from(native_error: EntryRejectedErr) -> Self {
        match native_error {
            EntryRejectedErr::AlreadyPresent(given) => {
                let reason = format!("Entry is already a member: {}", given.peer);
                APIErrorAndReason {
                    error: String::from(ENTRY_ALREADY_PRESENT_LABEL),
                    reason: String::from(reason),
                }
            }
            EntryRejectedErr::InvalidURL(given) => {
                let reason = format!("Entry URL is invalid: {}", given);
                APIErrorAndReason {
                    error: String::from(ENTRY_URL_INVALID_LABEL),
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
