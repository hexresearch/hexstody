mod runner;

use runner::run_test;

#[tokio::test]
async fn test_simple() {
    run_test(|env| async move {
        env.hot_client.ping().await.expect("Ping finished");
    })
    .await;
}
