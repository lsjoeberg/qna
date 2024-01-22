use handle_errors::Error;
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
    // FIXME: App configuration.
    let api_key = std::env::var("BADWORDS_API_KEY").expect("Env var BADWORDS_API_KEY must be set");
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.apilayer.com/bad_words?censor_character=*")
        .header("apikey", api_key)
        .body(content)
        .send()
        .await
        .map_err(Error::ExternalAPIError)?;

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
        Err(err) => Err(Error::ExternalAPIError(err)),
    }
}

async fn transform_error(res: reqwest::Response) -> handle_errors::APILayerError {
    handle_errors::APILayerError {
        status: res.status().as_u16(),
        message: res.json::<APIResponse>().await.unwrap().message,
    }
}
