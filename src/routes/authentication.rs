use reqwest::StatusCode;

use crate::account::Account;
use crate::store::Store;

pub async fn register(store: Store, account: Account) -> Result<impl warp::Reply, warp::Rejection> {
    match store.add_account(account).await {
        Ok(_) => Ok(warp::reply::with_status("Account created", StatusCode::OK)),
        Err(err) => Err(warp::reject::custom(err)),
    }
}
