mod runner;

use bitcoincore_rpc::{Client, RpcApi};
use bitcoin::Amount;

use runner::*;

// Get 50 BTC to the node wallet
async fn fund_wallet(client: &Client) {
    client.create_wallet("", None, None, None, None).expect("create default wallet");
    let address = client.get_new_address(None, None).expect("new address");
    client.generate_to_address(101, &address).expect("mined blocks");
}

#[tokio::test]
async fn basic_test() {
    run_test(|btc, api| async move { 
        println!("Running basic test");
        let info = btc.get_blockchain_info().expect("blockchain info");
        assert_eq!(info.chain, "regtest");
        api.ping().await.expect("API ping");
    }).await;
}

#[tokio::test]
async fn generate_test() {
    run_test(|btc, _| async move { 
        println!("Running generate test");
        fund_wallet(&btc).await;
        let balance = btc.get_balance(None, None).expect("balance");
        assert_eq!(balance, Amount::from_btc(50.0).unwrap());
    }).await;
}

#[tokio::test]
async fn deposit_test() {
    run_test(|btc, api| async move { 
        println!("Running simple deposit test");
        fund_wallet(&btc).await;
        
    }).await;
}