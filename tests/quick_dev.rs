use serde_json::json;
use anyhow::Result;

mod common;

#[tokio::test]
async fn quick_dev() -> Result<()> {

    common::setup().await;

    let hc = httpc_test::new_client("http://localhost:8080")?;
    let req_login = hc.do_post(
        "/api/login",
        json!({
            "username": "demo1",
            "pwd": "welcome"
        })
    );

    req_login.await?.print().await?;

    Ok(())
}
