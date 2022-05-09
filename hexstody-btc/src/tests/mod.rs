mod runner;

use bitcoin::{Address, Amount, Txid};
use bitcoincore_rpc::{Client, RpcApi};
use hexstody_btc_api::deposit::*;

use runner::*;

// Get 50 BTC to the node wallet
async fn fund_wallet(client: &Client) {
    let address = new_address(client);
    let mature_blocks = 100;
    client
        .generate_to_address(mature_blocks + 1, &address)
        .expect("mined blocks");
}

// Get a fresh address from the node
fn new_address(client: &Client) -> Address {
    client.get_new_address(None, None).expect("new address")
}

// Send mined btc to given address
fn send_funds(client: &Client, address: &Address, amount: Amount) -> Txid {
    client
        .send_to_address(address, amount, None, None, None, Some(true), None, None)
        .expect("funds sent")
}

#[tokio::test]
async fn basic_test() {
    run_test(|btc, api| async move {
        println!("Running basic test");
        let info = btc.get_blockchain_info().expect("blockchain info");
        assert_eq!(info.chain, "regtest");
        api.ping().await.expect("API ping");
    })
    .await;
}

#[tokio::test]
async fn generate_test() {
    run_test(|btc, _| async move {
        println!("Running generate test");
        fund_wallet(&btc).await;
        let balance = btc.get_balance(None, None).expect("balance");
        assert_eq!(balance, Amount::from_btc(50.0).unwrap());
    })
    .await;
}

#[tokio::test]
async fn deposit_test() {
    run_test(|btc, api| async move {
        println!("Running simple deposit test");
        fund_wallet(&btc).await;
        let deposit_address = new_address(&btc);
        let dep_txid = send_funds(&btc, &deposit_address, Amount::from_sat(1000));
        let res = api.deposit_events().await.expect("Deposit events");
        assert_eq!(res.events.len(), 1);
        let event = &res.events[0];
        if let DepositEvent::Update(DepositTxUpdate {txid, address, confirmations, ..}) = event {
            assert_eq!(txid.0, dep_txid);
            assert_eq!(address.0, deposit_address);
            assert_eq!(*confirmations, 0);
        } else {
            assert!(false, "Wrong type of event {:?}, expected deposit with txid {:?}", event, dep_txid);
        }
    })
    .await;
}
