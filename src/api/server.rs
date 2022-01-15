use crate::blockchain::block::{message_as_json, Block};
use crate::blockchain::{Chain, InvalidBlockErr};
use serde::{Deserialize, Serialize};
use crate::api::structs::{BlockList, Limits};
use std::sync::Once;
use std::sync::{Arc, Mutex};
use tide::{Body, Request, Response, Server, StatusCode};

static INIT: Once = Once::new();

#[derive(Clone)]
pub struct State {
    chain: Arc<Mutex<Chain>>,
}

impl State {
    fn new(genesis_data: String) -> Self {
        Self {
            chain: Arc::new(Mutex::new(Chain::new(genesis_data))),
        }
    }
    fn append_block(&self, block: Block) -> Result<Block, InvalidBlockErr> {
        let mut chain = self.chain.lock().unwrap();
        chain.append(block)
    }
}

#[async_std::main]
pub async fn main() -> tide::Result<()> {
    create_app(String::from(""))
        .listen("127.0.0.1:8080")
        .await?;
    Ok(())
}

async fn get_last_block(req: Request<State>) -> tide::Result<Response> {
    let state = req.state();
    let chain = &state.chain.lock().unwrap();
    let block: &Block = chain.get_last_block().unwrap();
    let mut res = Response::new(tide::StatusCode::Ok);
    res.set_body(Body::from_json(block)?);
    Ok(res)
}

async fn get_blocks(req: Request<State>) -> tide::Result<Response> {
    let limits: Limits = req.query()?;
    let state = req.state();

    let chain = &state.chain.lock().unwrap();
    let items: Vec<Block> = chain.blocks[limits.from_index..].iter().cloned().collect();
    let blocks = BlockList { items: items };
    let mut res = Response::new(tide::StatusCode::Ok);
    res.set_body(Body::from_json(&blocks)?);
    Ok(res)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct APIErrorAndReason {
    error: String,
    reason: String,
}

fn explain_error(error: Result<Block, InvalidBlockErr>) -> APIErrorAndReason {
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

async fn post_block(mut req: Request<State>) -> tide::Result<Response> {
    let block: Block = req.body_json().await?;
    let state = req.state();
    let added = state.append_block(block);

    match added {
        Ok(new_block) => {
            let mut res = Response::new(StatusCode::Ok);
            res.set_body(Body::from_json(&new_block)?);
            Ok(res)
        }
        error => {
            let mut res = Response::new(StatusCode::BadRequest);
            let error_and_reasion = explain_error(error);
            res.set_body(Body::from_json(&error_and_reasion)?);
            Ok(res)
        }
    }
}

pub fn create_app(genesis_data: String) -> Server<State> {
    INIT.call_once(tide::log::start);
    let mut app = tide::with_state(State::new(genesis_data));
    app.at("/blocks/last").get(get_last_block);
    app.at("/blocks").post(post_block).get(get_blocks);
    app
}

#[cfg(test)]
mod tests {

    use super::*;
    use tide::http::{Method, Request, Response, Url};

    fn arrange_second_block(app: &Server<State>) {
        let mut chain = app.state().chain.lock().unwrap();
        let first_block = &chain.blocks[0];
        let second = Block {
            index: 1,
            previous_hash: first_block.hash(),
            timestamp: first_block.timestamp + 100,
            data: message_as_json("Second block data"),
        };
        chain.append(second).unwrap();
    }

    async fn request_get_block(position: &str, app: &Server<State>) -> tide::Result<Response> {
        let block_url = &*format!("https://example.com/blocks/{position}", position = position);
        let url = Url::parse(block_url).unwrap();
        let req = Request::new(Method::Get, url);
        let res: Response = app.respond(req).await?;
        Ok(res)
    }

    async fn request_get_blocks(limits: &str, app: &Server<State>) -> tide::Result<Response> {
        let block_url = &*format!("https://example.com/blocks?{limits}", limits = limits);
        let url = Url::parse(block_url).unwrap();
        let req = Request::new(Method::Get, url);
        let res: Response = app.respond(req).await?;
        Ok(res)
    }

    async fn request_post_block(block: Block, app: &Server<State>) -> tide::Result<Response> {
        let block_url = String::from("https://example.com/blocks");
        let url = Url::parse(&*block_url).unwrap();
        let mut req = Request::new(Method::Post, url);
        let content = serde_json::to_string(&block).unwrap();
        req.set_body(content);
        let res: Response = app.respond(req).await?;
        Ok(res)
    }

    async fn get_block_from_server_status(app: &Server<State>, index: u32) -> Block {
        let chain = &app.state().chain.lock().unwrap();
        chain.blocks[index as usize].clone()
    }

    async fn block_from_body(mut response: Response) -> Result<Block, serde_json::Error> {
        let data = response.body_string().await.unwrap();
        serde_json::from_str(&*data)
    }

    async fn block_list_from_body(mut response: Response) -> Result<BlockList, serde_json::Error> {
        let data = response.body_string().await.unwrap();
        serde_json::from_str(&*data)
    }

    async fn error_from_body(
        mut response: Response,
    ) -> Result<APIErrorAndReason, serde_json::Error> {
        let data = response.body_string().await.unwrap();
        serde_json::from_str(&*data)
    }

    #[async_std::test]
    async fn get_last_block_being_genesis() -> tide::Result<()> {
        let app = create_app(String::from("Genesis block sample"));
        let confirmation: Response = request_get_block("last", &app).await?;
        let received_block: Block = block_from_body(confirmation).await?;
        assert_eq!(0, received_block.index);
        assert_eq!(
            "Genesis block sample",
            received_block.data.get("message").unwrap()
        );
        assert_eq!("", received_block.previous_hash);
        Ok(())
    }

    #[async_std::test]
    async fn get_first_block_being_genesis() -> tide::Result<()> {
        let app = create_app(String::from("Genesis block sample"));
        let confirmation = request_get_blocks("from_index=0", &app).await?;
        let received_list: BlockList = block_list_from_body(confirmation).await?;
        assert_eq!(1, received_list.items.len());
        let received_block: Block = received_list.items[0].clone();
        assert_eq!(
            "Genesis block sample",
            received_block.data.get("message").unwrap()
        );
        assert_eq!("", received_block.previous_hash);
        assert_eq!(0, received_block.index);
        Ok(())
    }

    #[async_std::test]
    async fn get_last_block_being_second() -> tide::Result<()> {
        let app = create_app(String::from("Genesis block sample"));
        arrange_second_block(&app);
        let confirmation = request_get_block("last", &app).await?;
        let received_block = block_from_body(confirmation).await?;
        assert_eq!(1, received_block.index);
        assert_eq!(
            "Second block data",
            received_block.data.get("message").unwrap()
        );
        Ok(())
    }

    #[async_std::test]
    async fn get_block_one_being_list_first() -> tide::Result<()> {
        let app = create_app(String::from("Genesis block sample"));
        arrange_second_block(&app);
        let confirmation = request_get_blocks("from_index=1", &app).await?;
        let received_list: BlockList = block_list_from_body(confirmation).await?;
        let obtained_block: Block = received_list.items[0].clone();
        assert_eq!(1, obtained_block.index);
        assert_eq!(
            "Second block data",
            obtained_block.data.get("message").unwrap()
        );
        Ok(())
    }

    #[async_std::test]
    async fn get_genesis_block_being_list_first() -> tide::Result<()> {
        let app = create_app(String::from("Genesis block sample"));
        arrange_second_block(&app);
        let confirmation = request_get_blocks("from_index=0", &app).await?;
        let received_list: BlockList = block_list_from_body(confirmation).await?;
        let obtained_block: Block = received_list.items[0].clone();
        assert_eq!(2, received_list.items.len());
        assert_eq!(0, obtained_block.index);
        assert_eq!(
            "Genesis block sample",
            obtained_block.data.get("message").unwrap()
        );
        Ok(())
    }

    #[async_std::test]
    async fn get_no_blocks_from_one() -> tide::Result<()> {
        let app = create_app(String::from("Genesis block sample"));
        //arrange_second_block(&app);
        let confirmation = request_get_blocks("from_index=1", &app).await?;
        let received_list: BlockList = block_list_from_body(confirmation).await?;
        assert_eq!(0, received_list.items.len());
        Ok(())
    }

    #[async_std::test]
    async fn post_new_block_results_ok() -> tide::Result<()> {
        let app = create_app(String::from("Genesis block sample"));
        let first_block = get_block_from_server_status(&app, 0).await;
        let second = Block {
            index: 1,
            previous_hash: first_block.hash(),
            timestamp: first_block.timestamp + 100,
            data: message_as_json("Second block data"),
        };
        let confirmation = request_post_block(second, &app).await?;
        let confirmed_block = block_from_body(confirmation).await?;
        assert_eq!(1, confirmed_block.index);
        assert_eq!(
            "Second block data",
            confirmed_block.data.get("message").unwrap()
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_fails_to_append_by_hash() -> tide::Result<()> {
        let app = create_app(String::from("Genesis block sample"));
        let first_block = get_block_from_server_status(&app, 0).await;
        let second = Block {
            index: 1,
            previous_hash: String::from("c4f3c4f3c4f3"),
            timestamp: first_block.timestamp + 100,
            data: message_as_json("Second block data"),
        };
        let expected_reason = format!(
            "previous hash is {} but {} was provided",
            first_block.hash(),
            second.previous_hash
        );
        let confirmation = request_post_block(second, &app).await?;
        let confirmation_status = confirmation.status();
        let report = error_from_body(confirmation).await?;
        assert_eq!(400, confirmation_status);
        assert_eq!(String::from("Previous hash not matching"), report.error);
        assert_eq!(String::from(expected_reason), report.reason);
        Ok(())
    }

    #[async_std::test]
    async fn test_fails_to_append_by_index() -> tide::Result<()> {
        let app = create_app(String::from("Genesis block sample"));
        let first_block = get_block_from_server_status(&app, 0).await;
        let second = Block {
            index: 3,
            previous_hash: first_block.hash(),
            timestamp: first_block.timestamp + 100,
            data: message_as_json("Second block data"),
        };
        let expected_reason = "expected index 0 but received 3 which is not inmediate next";
        let confirmation = request_post_block(second, &app).await?;
        let confirmation_status = confirmation.status();
        let report = error_from_body(confirmation).await?;
        assert_eq!(400, confirmation_status);
        assert_eq!(
            String::from("New block index is not correlative"),
            report.error
        );
        assert_eq!(String::from(expected_reason), report.reason);
        Ok(())
    }

    #[async_std::test]
    async fn test_fails_to_append_by_timestamp() -> tide::Result<()> {
        let app = create_app(String::from("Genesis block sample"));
        let first_block = get_block_from_server_status(&app, 0).await;
        let second = Block {
            index: 1,
            previous_hash: first_block.hash(),
            timestamp: first_block.timestamp - 100,
            data: message_as_json("Second block data"),
        };
        let expected_reason = format!(
            "Given timestamp {} is not later to {}",
            second.timestamp, first_block.timestamp
        );
        let confirmation = request_post_block(second, &app).await?;
        let confirmation_status = confirmation.status();
        let report = error_from_body(confirmation).await?;
        assert_eq!(400, confirmation_status);
        assert_eq!(
            String::from("New block timestamp must be later to previous"),
            report.error
        );
        assert_eq!(String::from(expected_reason), report.reason);
        Ok(())
    }
}
