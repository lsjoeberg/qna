use std::env;

use handle_errors::Error;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct APIResponse {
    message: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BadWord {
    original: String,
    word: String,
    deviations: i64,
    info: i64,
    #[serde(rename = "replacedLen")]
    replace_len: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BadWordResponse {
    content: String,
    bad_words_total: i64,
    bad_words_list: Vec<BadWord>,
    censored_content: String,
}

pub async fn check_profanity(content: String) -> Result<String, Error> {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();
    let api_key = env::var("BADWORDS_API_KEY").expect("Env var BADWORDS_API_KEY must be set");
    let api_layer_url = env::var("BADWORDS_URL").expect("Env var BADWORDS_URL must be set");
    let res = client
        .post(format!("{}/bad_words?censor_character=*", api_layer_url))
        .header("apikey", api_key)
        .body(content)
        .send()
        .await
        .map_err(Error::MiddlewareReqwestAPIError)?;

    if !res.status().is_success() {
        return if res.status().is_client_error() {
            let err = transform_error(res).await;
            Err(Error::ClientError(err))
        } else {
            let err = transform_error(res).await;
            Err(Error::ServerError(err))
        };
    }

    match res.json::<BadWordResponse>().await {
        Ok(res) => Ok(res.censored_content),
        Err(err) => Err(Error::ReqwestAPIError(err)),
    }
}

async fn transform_error(res: reqwest::Response) -> handle_errors::APILayerError {
    handle_errors::APILayerError {
        status: res.status().as_u16(),
        message: res.json::<APIResponse>().await.unwrap().message,
    }
}
#[cfg(test)]
mod profanity_tests {
    use mock_server::{MockServer, OneshotHandler};

    use super::{check_profanity, env};

    #[tokio::test]
    async fn run() {
        let handler = run_mock();
        censor_profane_words().await;
        no_profane_words().await;
        let _ = handler.sender.send(1);
    }

    fn run_mock() -> OneshotHandler {
        env::set_var("BADWORDS_URL", "http://127.0.0.1:3030");
        env::set_var("BADWORDS_API_KEY", "YES");

        let socket = "127.0.0.1:3030"
            .to_string()
            .parse()
            .expect("Not a valid address");
        let mock = MockServer::new(socket);

        mock.oneshot()
    }

    async fn censor_profane_words() {
        let content = "This is a shitty sentence".to_string();
        let censored_content = check_profanity(content).await;
        assert_eq!(censored_content.unwrap(), "this is a ****** sentence");
    }

    async fn no_profane_words() {
        let content = "this is a sentence".to_string();
        let censored_content = check_profanity(content).await;
        assert_eq!(censored_content.unwrap(), "");
    }
}
