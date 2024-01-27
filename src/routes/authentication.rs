use std::future;

use argon2::Config;
use chrono::Utc;
use pasetors::{
    claims::{Claims, ClaimsValidationRules},
    keys::SymmetricKey,
    token::UntrustedToken,
    version4::V4,
    Local,
};
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

    let key = SymmetricKey::<V4>::from(key.as_bytes()).expect("Invalid PASETO_KEY");
    let mut claims = Claims::new().unwrap();
    claims
        .add_additional("account_id", serde_json::json!(account_id))
        .expect("Failed to set claim 'account_id'");
    claims
        .expiration(&dt.to_rfc3339())
        .expect("Failed to set claim 'exp'");
    claims
        .not_before(&Utc::now().to_rfc3339())
        .expect("Failed to set claim 'nbf'");
    pasetors::local::encrypt(&key, &claims, None, None)
        .expect("Failed to construct paseto token with builder!")
}

pub fn verify_token(token: String) -> Result<Session, handle_errors::Error> {
    let key = std::env::var("PASETO_KEY").expect("Env var PASETO_KEY must be set");
    let key = SymmetricKey::<V4>::from(key.as_bytes()).expect("Invalid PASETO_KEY");
    let validation_rules = ClaimsValidationRules::new();
    let untrusted_token = UntrustedToken::<Local, V4>::try_from(&token)
        .map_err(|_| handle_errors::Error::CannotDecryptToken)?;
    let trusted_token =
        pasetors::local::decrypt(&key, &untrusted_token, &validation_rules, None, None)
            .map_err(|_| handle_errors::Error::CannotDecryptToken)?;

    serde_json::from_str::<Session>(trusted_token.payload())
        .map_err(|_| handle_errors::Error::CannotDecryptToken)
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
