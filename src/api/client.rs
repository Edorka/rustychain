use crate::api::errors::APIErrorAndReason;
use crate::peers::{EntryRejectedErr, MemberEntry};
use crate::api::structs::{BlockList, Limits};
use crate::blockchain::block::{get_epoch_ms, message_as_json, Block};
use crate::blockchain::InvalidBlockErr;
use surf::{Error, Response};

struct APIClient {
    host_url: String,
}

impl APIClient {
    fn new(host_url: String) -> Self {
        Self { host_url: host_url }
    }
    async fn get_all_blocks(&self) -> Result<BlockList, Error> {
        let mut response: Response = surf::get(format!("{}/blocks", &self.host_url))
            .await
            .unwrap();
        let list: BlockList = response.body_json().await?;
        Ok(list)
    }
    async fn get_blocks(&self, from_index: usize) -> Result<BlockList, Error> {
        let limits = Limits {
            from_index: from_index,
        };
        let mut response: Response =
            surf::get(format!("{}/blocks?{}", &self.host_url, limits.as_query()))
                .await
                .unwrap();
        let list: BlockList = response.body_json().await?;
        Ok(list)
    }
    async fn send_block(&self, block: Block) -> Result<Block, InvalidBlockErr> {
        let mut response: Response = surf::post(format!("{}/blocks", &self.host_url))
            .body_json(&block)
            .unwrap()
            .await
            .unwrap();
        match response.status().is_success() {
            true => {
                let confirmed: Block = response.body_json().await.unwrap();
                Ok(confirmed)
            }
            _ => {
                let api_error: APIErrorAndReason = response.body_json().await.unwrap();
                let error: InvalidBlockErr = api_error.into();
                Err(error)
            }
        }
    }
    async fn send_peer(&self, peer: MemberEntry) -> Result<MemberEntry, EntryRejectedErr> {
        let mut response: Response = surf::post(format!("{}/peers", &self.host_url))
            .body_json(&peer)
            .unwrap()
            .await
            .unwrap();
        match response.status().is_success() {
            true => {
                let confirmed: MemberEntry = response.body_json().await.unwrap();
                Ok(confirmed)
            }
            _ => {
                let error: APIErrorAndReason = response.body_json().await.unwrap();
                let nat_error: EntryRejectedErr = error.into();
                match nat_error {
                    EntryRejectedErr::AlreadyPresent(confirmed) => Ok(confirmed),
                    _ => Err(nat_error),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::blockchain::InvalidBlockErr;
    use wiremock::http::Method;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // Start a background HTTP server on a random local port
    async fn arrange_server_mock_get_blocks(blocks: Option<Vec<Block>>) -> MockServer {
        let items = match blocks {
            None => [].to_vec(),
            Some(blocks) => blocks,
        };
        let mock_server = MockServer::start().await;
        let sample: BlockList = BlockList { items: items };

        // Arrange the behaviour of the MockServer adding a Mock:
        Mock::given(method("GET"))
            .and(path("/blocks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample))
            // Mounting the mock on the mock server - it's now effective!
            .mount(&mock_server)
            .await;
        mock_server
    }

    // Start a background HTTP server on a random local port
    async fn arrange_server_mock_receive_block(block: Block) -> MockServer {
        let mock_server = MockServer::start().await;

        // Arrange the behaviour of the MockServer adding a Mock:
        Mock::given(method("POST"))
            .and(path("/blocks"))
            .respond_with(ResponseTemplate::new(201).set_body_json(block))
            // Mounting the mock on the mock server - it's now effective!
            .mount(&mock_server)
            .await;
        mock_server
    }

    async fn arrange_server_mock_receive_peer(peer: MemberEntry) -> MockServer {
        let mock_server = MockServer::start().await;

        // Arrange the behaviour of the MockServer adding a Mock:
        Mock::given(method("POST"))
            .and(path("/peers"))
            .respond_with(ResponseTemplate::new(201).set_body_json(peer))
            // Mounting the mock on the mock server - it's now effective!
            .mount(&mock_server)
            .await;
        mock_server
    }

    async fn arrange_server_mock_reject_block(error: APIErrorAndReason) -> MockServer {
        let mock_server = MockServer::start().await;

        // Arrange the behaviour of the MockServer adding a Mock:
        Mock::given(method("POST"))
            .and(path("/blocks"))
            .respond_with(ResponseTemplate::new(400).set_body_json(error))
            // Mounting the mock on the mock server - it's now effective!
            .mount(&mock_server)
            .await;
        mock_server
    }

    async fn arrange_server_mock_reject_peer(error: APIErrorAndReason) -> MockServer {
        let mock_server = MockServer::start().await;

        // Arrange the behaviour of the MockServer adding a Mock:
        Mock::given(method("POST"))
            .and(path("/peers"))
            .respond_with(ResponseTemplate::new(400).set_body_json(error))
            // Mounting the mock on the mock server - it's now effective!
            .mount(&mock_server)
            .await;
        mock_server
    }

    #[async_std::test]
    async fn get_all_blocks_being_empty() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = arrange_server_mock_get_blocks(None).await;
        let client = APIClient::new(mock_server.uri());
        let list = client.get_all_blocks().await?;
        assert_eq!(list.items.len(), 0);
        Ok(())
    }

    #[async_std::test]
    async fn get_all_blocks_being_two() -> Result<(), Box<dyn std::error::Error>> {
        // Start a background HTTP server on a random local port
        let genesis_block = Block {
            index: 0,
            previous_hash: String::from(""),
            timestamp: get_epoch_ms(),
            data: message_as_json("Genesis block"),
        };
        let second_block = Block {
            index: 1,
            previous_hash: genesis_block.hash(),
            timestamp: genesis_block.timestamp + 100,
            data: message_as_json("Second block data"),
        };
        let items: Vec<Block> = [genesis_block, second_block].to_vec();
        let mock_server = arrange_server_mock_get_blocks(Some(items)).await;

        let client = APIClient::new(mock_server.uri());
        let list = client.get_all_blocks().await?;
        assert_eq!(list.items.len(), 2);
        Ok(())
    }

    #[async_std::test]
    async fn get_blocks_from_index_1() -> Result<(), Box<dyn std::error::Error>> {
        // Start a background HTTP server on a random local port
        let genesis_block = Block {
            index: 0,
            previous_hash: String::from(""),
            timestamp: get_epoch_ms(),
            data: message_as_json("Genesis block"),
        };
        let second_block = Block {
            index: 1,
            previous_hash: genesis_block.hash(),
            timestamp: genesis_block.timestamp + 100,
            data: message_as_json("Second block data"),
        };
        let items: Vec<Block> = [genesis_block, second_block].to_vec();
        let mock_server = arrange_server_mock_get_blocks(Some(items)).await;

        let client = APIClient::new(mock_server.uri());
        let list = client.get_blocks(1).await?;
        let received_requests = mock_server.received_requests().await.unwrap();
        let received_request = &received_requests[0];
        assert_eq!(list.items.len(), 2);
        assert_eq!(received_requests.len(), 1);
        assert_eq!(received_request.url.query().unwrap(), "from_index=1");
        assert_eq!(received_request.method, Method::Get);
        assert!(received_request.body.is_empty());
        Ok(())
    }

    #[async_std::test]
    async fn test_sent_block_accepted() -> Result<(), Box<dyn std::error::Error>> {
        // Start a background HTTP server on a random local port
        let second_block = Block {
            index: 1,
            previous_hash: String::from("not important"),
            timestamp: get_epoch_ms(),
            data: message_as_json("Second block data"),
        };
        let mock_server = arrange_server_mock_receive_block(second_block.clone()).await;

        let client = APIClient::new(mock_server.uri());

        let expected_second_block = second_block.clone();
        let result = client.send_block(second_block).await.unwrap();
        let received_requests = mock_server.received_requests().await.unwrap();
        let received_request = &received_requests[0];
        assert_eq!(result, expected_second_block);
        assert_eq!(received_requests.len(), 1);
        assert_eq!(received_request.method, Method::Post);
        Ok(())
    }

    #[async_std::test]
    async fn test_sent_block_rejected_because_hash() -> Result<(), ()> {
        // Start a background HTTP server on a random local port
        let error = InvalidBlockErr::HashNotMatching(
            String::from("00000000000000000000000000000000"),
            String::from("11111111111111111111111111111111"),
        );
        let api_error = APIErrorAndReason::from(error.clone());

        let second_block = Block {
            index: 0,
            previous_hash: String::from("reallydoesntmatter"),
            timestamp: get_epoch_ms(),
            data: message_as_json("Sample second block"),
        };
        let mock_server = arrange_server_mock_reject_block(api_error).await;

        let client = APIClient::new(mock_server.uri());

        let failure = client.send_block(second_block).await.unwrap_err();
        let received_requests = mock_server.received_requests().await.unwrap();
        let received_request = &received_requests[0];
        assert_eq!(received_requests.len(), 1);
        assert_eq!(received_request.method, Method::Post);
        assert_eq!(failure, error);
        Ok(())
    }

    #[async_std::test]
    async fn test_sent_block_rejected_because_timestamp() -> Result<(), ()> {
        // Start a background HTTP server on a random local port
        let error = InvalidBlockErr::NotPosterior(1000, 2000);
        let api_error = APIErrorAndReason::from(error.clone());

        let second_block = Block {
            index: 0,
            previous_hash: String::from("reallydoesntmatter"),
            timestamp: get_epoch_ms(),
            data: message_as_json("Sample second block"),
        };
        let mock_server = arrange_server_mock_reject_block(api_error).await;

        let client = APIClient::new(mock_server.uri());

        let failure = client.send_block(second_block).await.unwrap_err();
        let received_requests = mock_server.received_requests().await.unwrap();
        let received_request = &received_requests[0];
        assert_eq!(received_requests.len(), 1);
        assert_eq!(received_request.method, Method::Post);
        assert_eq!(failure, error);
        Ok(())
    }

    #[async_std::test]
    async fn test_sent_block_rejected_because_index() -> Result<(), ()> {
        // Start a background HTTP server on a random local port
        let error = InvalidBlockErr::NotCorrelated(1, 2);
        let api_error: APIErrorAndReason = APIErrorAndReason::from(error.clone());

        let second_block = Block {
            index: 0,
            previous_hash: String::from("reallydoesntmatter"),
            timestamp: get_epoch_ms(),
            data: message_as_json("Sample second block"),
        };
        let mock_server = arrange_server_mock_reject_block(api_error).await;

        let client = APIClient::new(mock_server.uri());

        let failure = client.send_block(second_block).await.unwrap_err();
        let received_requests = mock_server.received_requests().await.unwrap();
        let received_request = &received_requests[0];
        assert_eq!(received_requests.len(), 1);
        assert_eq!(received_request.method, Method::Post);
        assert_eq!(failure, error);
        Ok(())
    }

    #[async_std::test]
    async fn test_sent_peer_accepted() -> Result<(), Box<dyn std::error::Error>> {
        // Start a background HTTP server on a random local port
        let new_member = MemberEntry {
            peer: String::from("ws://localhost:5055"),
        };
        let mock_server = arrange_server_mock_receive_peer(new_member.clone()).await;
        let client = APIClient::new(mock_server.uri());

        let expected_new_member = new_member.clone();
        let result: MemberEntry = client.send_peer(new_member).await.unwrap();
        let received_requests = mock_server.received_requests().await.unwrap();
        let received_request = &received_requests[0];
        assert_eq!(result, expected_new_member);
        assert_eq!(received_requests.len(), 1);
        assert_eq!(received_request.method, Method::Post);
        Ok(())
    }

    #[async_std::test]
    async fn test_sent_peer_rejected_malformed() -> Result<(), Box<dyn std::error::Error>> {
        // Start a background HTTP server on a random local port
        let url = String::from("ws://localhost:5055");
        let new_member = MemberEntry {
            peer: url.clone()
        };
        let error = EntryRejectedErr::InvalidURL(url.clone());
        let api_error: APIErrorAndReason = APIErrorAndReason::from(error.clone());
        let mock_server = arrange_server_mock_reject_peer(api_error).await;
        let client = APIClient::new(mock_server.uri());

        let failure = client.send_peer(new_member).await.unwrap_err();
        let received_requests = mock_server.received_requests().await.unwrap();
        let received_request = &received_requests[0];
        assert!(matches!(failure, EntryRejectedErr::InvalidURL(error_url) if url == error_url));
        assert_eq!(received_requests.len(), 1);
        assert_eq!(received_request.method, Method::Post);
        Ok(())
    }

    #[async_std::test]
    async fn test_sent_peer_already_present() -> Result<(), Box<dyn std::error::Error>> {
        // Start a background HTTP server on a random local port
        let new_member = MemberEntry {
            peer: String::from("ws://localhost:5055"),
        };
        let mock_server = arrange_server_mock_receive_peer(new_member.clone()).await;
        let client = APIClient::new(mock_server.uri());

        let expected_new_member = new_member.clone();
        let result: MemberEntry = client.send_peer(new_member).await.unwrap();
        let received_requests = mock_server.received_requests().await.unwrap();
        let received_request = &received_requests[0];
        assert_eq!(result, expected_new_member);
        assert_eq!(received_requests.len(), 1);
        assert_eq!(received_request.method, Method::Post);
        Ok(())
    }
}
