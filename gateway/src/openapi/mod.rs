mod response;
mod router;

use rocket::{get, post};

#[post("/v1")]
pub async fn call() {
    return;
}
