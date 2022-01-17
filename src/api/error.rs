use crate::blockchain::block::Block;
use crate::blockchain::InvalidBlockErr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct APIErrorAndReason {
    pub error: String,
    pub reason: String,
}

pub fn explain_error(error: Result<Block, InvalidBlockErr>) -> APIErrorAndReason {
    match error.unwrap_err() {
        InvalidBlockErr::HashNotMatching(given, expected) => {
            let reason = format!("previous hash is {} but {} was provided", expected, given);
            APIErrorAndReason {
                error: String::from("Previous hash not matching"),
                reason: String::from(reason),
            }
        }
        InvalidBlockErr::NotCorrelated(given, expected) => {
            let reason = format!(
                "expected index {} but received {} which is not inmediate next",
                expected, given
            );
            APIErrorAndReason {
                error: String::from("New block index is not correlative"),
                reason: String::from(reason),
            }
        }
        InvalidBlockErr::NotPosterior(given, expected) => {
            let reason = format!("Given timestamp {} is not later to {}", given, expected);
            APIErrorAndReason {
                error: String::from("New block timestamp must be later to previous"),
                reason: String::from(reason),
            }
        }
        _ => APIErrorAndReason {
            error: String::from("Unknown error"),
            reason: String::from("reason"),
        },
    }
}
