use argon2::Config;
use chrono::Utc;
use rand::random;
use reqwest::StatusCode;

use crate::account::{Account, AccountId};
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
    let key = std::env::var("PASETO_KEY").expect("Env var PASETO_KEY must be set"); // FIXME

    paseto::tokens::PasetoBuilder::new()
        .set_encryption_key(key.as_bytes())
        .set_expiration(&dt)
        .set_not_before(&Utc::now())
        .set_claim("account_id", serde_json::json!(account_id))
        .build()
        .expect("Failed to construct paseto token with builder!")
}
