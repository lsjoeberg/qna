use std::future;

use argon2::Config;
use chrono::Utc;
use rand::random;
use reqwest::StatusCode;
use warp::Filter;

use crate::account::{Account, AccountId, Session};
use crate::store::Store;

pub async fn register(store: Store, account: Account) -> Result<impl warp::Reply, warp::Rejection> {
    let hashed_password = hash_password(account.password.as_bytes());
    let account = Account {
        id: account.id,
        email: account.email,
        password: hashed_password,
    };

    match store.add_account(account).await {
        Ok(_) => Ok(warp::reply::with_status("Account created", StatusCode::OK)),
        Err(err) => Err(warp::reject::custom(err)),
    }
}

pub async fn login(store: Store, login: Account) -> Result<impl warp::Reply, warp::Rejection> {
    let result = store.get_account(login.email).await;
    match result {
        Ok(account) => match verify_password(&account.password, login.password.as_bytes()) {
            Ok(verified) => {
                if verified {
                    Ok(warp::reply::json(&issue_token(
                        account.id.expect("id not found"),
                    )))
                } else {
                    Err(warp::reject::custom(handle_errors::Error::WrongPassword))
                }
            }
            Err(err) => Err(warp::reject::custom(
                handle_errors::Error::ArgonLibraryError(err),
            )),
        },
        Err(err) => Err(warp::reject::custom(err)),
    }
}

pub fn hash_password(password: &[u8]) -> String {
    let salt = random::<[u8; 32]>();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config).unwrap()
}

fn verify_password(hash: &str, password: &[u8]) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(hash, password)
}

fn issue_token(account_id: AccountId) -> String {
    let current_date_time = Utc::now();
    let dt = current_date_time + chrono::Duration::days(1);
    let key = std::env::var("PASETO_KEY").expect("Env var PASETO_KEY must be set");

    paseto::tokens::PasetoBuilder::new()
        .set_encryption_key(key.as_bytes())
        .set_expiration(&dt)
        .set_not_before(&Utc::now())
        .set_claim("account_id", serde_json::json!(account_id))
        .build()
        .expect("Failed to construct paseto token with builder!")
}

pub fn verify_token(token: String) -> Result<Session, handle_errors::Error> {
    let key = std::env::var("PASETO_KEY").expect("Env var PASETO_KEY must be set");
    let token = paseto::tokens::validate_local_token(
        &token,
        None,
        key.as_bytes(),
        &paseto::tokens::TimeBackend::Chrono,
    )
    .map_err(|_| handle_errors::Error::CannotDecryptToken)?;
    serde_json::from_value::<Session>(token).map_err(|_| handle_errors::Error::CannotDecryptToken)
}

pub fn auth() -> impl Filter<Extract = (Session,), Error = warp::Rejection> + Clone {
    warp::header::<String>("Authorization").and_then(|token: String| {
        let token = match verify_token(token) {
            Ok(t) => t,
            Err(_) => return future::ready(Err(warp::reject::reject())),
        };
        future::ready(Ok(token))
    })
}
