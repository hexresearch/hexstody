use bitcoin::Amount;
use bitcoincore_rpc::RpcApi;
use hexstody_api::domain::CurrencyAddress;
use hexstody_api::types::{
    ConfirmedWithdrawal,
    WithdrawalRequestDecisionType, WithdrawalSignature
};
use hexstody_btc_api::bitcoin::*;
use hexstody_btc_api::events::*;
use hexstody_btc_test::helpers::*;
use hexstody_btc_test::runner::*;
use p256::ecdsa::SigningKey;
use p256::ecdsa::signature::Signer;
use rocket::serde::json;

// Check that we have node and API operational
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

// Check if we have balance after generating blocks
#[tokio::test]
async fn generate_test() {
    run_test(|btc, _| async move {
        println!("Running generate test");
        fund_wallet(&btc);
        let balance = btc.get_balance(None, None).expect("balance");
        assert_eq!(balance, Amount::from_btc(50.0).unwrap());
    })
    .await;
}

// Deposit unconfirmed transation
#[tokio::test]
async fn deposit_unconfirmed_test() {
    run_test(|btc, api| async move {
        println!("Running simple deposit test");
        fund_wallet(&btc);
        let deposit_address = new_address(&btc);
        let dep_txid = send_funds(&btc, &deposit_address, Amount::from_sat(1000));
        let res = api.poll_events().await.expect("Poll events");
        assert_eq!(res.events.len(), 2);
        let event = &res.events[1];
        if let BtcEvent::Update(TxUpdate {
            direction,
            txid,
            address,
            confirmations,
            ..
        }) = event
        {
            assert_eq!(*direction, TxDirection::Deposit);
            assert_eq!(txid.0, dep_txid);
            assert_eq!(address.0, deposit_address);
            assert_eq!(*confirmations, 0);
        } else {
            assert!(
                false,
                "Wrong type of event {:?}, expected deposit with txid {:?}",
                event, dep_txid
            );
        }
    })
    .await;
}

// Deposit confirmation transation
#[tokio::test]
async fn deposit_confirmed_test() {
    run_test(|btc, api| async move {
        println!("Running deposit confirmation test");
        fund_wallet(&btc);
        let deposit_address = new_address(&btc);
        let dep_txid = send_funds(&btc, &deposit_address, Amount::from_sat(1000));
        mine_blocks(&btc, 1);
        let res = api.poll_events().await.expect("Poll events");
        assert_eq!(res.events.len(), 2);
        let event = &res.events[1];
        if let BtcEvent::Update(TxUpdate {
            direction,
            txid,
            address,
            confirmations,
            ..
        }) = event
        {
            assert_eq!(*direction, TxDirection::Deposit);
            assert_eq!(txid.0, dep_txid);
            assert_eq!(address.0, deposit_address);
            assert_eq!(*confirmations, 1);
        } else {
            assert!(
                false,
                "Wrong type of event {:?}, expected deposit with txid {:?}",
                event, dep_txid
            );
        }
    })
    .await;
}

// Deposit transation and wait for next block after confirmation
#[tokio::test]
async fn deposit_confirmed_several_test() {
    run_test(|btc, api| async move {
        println!("Running simple deposit test");
        fund_wallet(&btc);
        let deposit_address = new_address(&btc);
        let _ = send_funds(&btc, &deposit_address, Amount::from_sat(1000));
        let height = btc.get_block_count().expect("block count");
        mine_blocks(&btc, 1);
        let res = api.poll_events().await.expect("Poll events");
        assert_eq!(res.events.len(), 2);
        mine_blocks(&btc, 1);
        let res = api.poll_events().await.expect("Poll events");
        assert_eq!(res.events.len(), 0);
        assert_eq!(res.height, height + 2);
    })
    .await;
}

// Deposit unconfirmed transation and cancel it
#[tokio::test]
async fn cancel_unconfirmed_test() {
    run_test(|btc, api| async move {
        println!("Cancel unconfirmed transaction test");
        fund_wallet(&btc);
        let deposit_address = new_address(&btc);
        let dep_txid = send_funds(&btc, &deposit_address, Amount::from_sat(1000));
        let res = api.poll_events().await.expect("Poll events");
        assert_eq!(res.events.len(), 2);

        let bumped_res = bumpfee(&btc, &dep_txid, None, None, None, None).expect("bump fee");
        let res = api.poll_events().await.expect("Poll events");
        assert_eq!(res.events.len(), 4, "Unexpected events: {:?}", res.events);

        mine_blocks(&btc, 1);
        let res = api.poll_events().await.expect("Poll events");
        assert_eq!(res.events.len(), 2, "Unexpected events: {:?}", res.events);

        let event = &res.events[0];
        if let BtcEvent::Update(TxUpdate {
            direction,
            txid,
            address,
            confirmations,
            conflicts,
            ..
        }) = event
        {
            assert_eq!(*direction, TxDirection::Withdraw);
            assert_eq!(txid.0, bumped_res.txid);
            assert_eq!(conflicts, &vec![BtcTxid(dep_txid)]);
            assert_eq!(address.0, deposit_address);
            assert_eq!(*confirmations, 1);
        } else {
            assert!(
                false,
                "Wrong type of event {:?}, expected deposit with txid {:?}",
                event, dep_txid
            );
        }

        let event = &res.events[1];
        if let BtcEvent::Update(TxUpdate {
            direction,
            txid,
            address,
            confirmations,
            conflicts,
            ..
        }) = event
        {
            assert_eq!(*direction, TxDirection::Deposit);
            assert_eq!(txid.0, bumped_res.txid);
            assert_eq!(conflicts, &vec![BtcTxid(dep_txid)]);
            assert_eq!(address.0, deposit_address);
            assert_eq!(*confirmations, 1);
        } else {
            assert!(
                false,
                "Wrong type of event {:?}, expected deposit with txid {:?}",
                event, dep_txid
            );
        }
    })
    .await;
}

// Deposit confirmed transation and cancel it
#[tokio::test]
async fn cancel_confirmed_test() {
    run_test(|btc, api| async move {
        println!("Cancel confirmed transaction test");
        fund_wallet(&btc);
        let deposit_address = new_address(&btc);
        let dep_txid = send_funds(&btc, &deposit_address, Amount::from_sat(1000));

        mine_blocks(&btc, 1);
        let res = api.poll_events().await.expect("Poll events");
        assert_eq!(res.events.len(), 2);

        let last_block = btc.get_best_block_hash().expect("best block");
        btc.invalidate_block(&last_block).expect("forget block");

        let res = api.poll_events().await.expect("Poll events");
        assert_eq!(res.events.len(), 3, "Unexpected events: {:?}", res.events);

        let event = &res.events[0];
        if let BtcEvent::Cancel(TxCancel {
            direction,
            txid,
            address,
            ..
        }) = event
        {
            assert_eq!(*direction, TxDirection::Deposit);
            assert_eq!(txid.0, dep_txid);
            assert_eq!(address.0, deposit_address);
        } else {
            assert!(
                false,
                "Wrong type of event {:?}, expected deposit with txid {:?}",
                event, dep_txid
            );
        }

        let event = &res.events[1];
        if let BtcEvent::Update(TxUpdate {
            direction,
            txid,
            address,
            confirmations,
            ..
        }) = event
        {
            assert_eq!(*direction, TxDirection::Withdraw);
            assert_eq!(txid.0, dep_txid);
            assert_eq!(address.0, deposit_address);
            assert_eq!(
                *confirmations, 0,
                "Expected confirmation counter is 0 after cancel"
            )
        } else {
            assert!(
                false,
                "Wrong type of event {:?}, expected deposit with txid {:?}",
                event, dep_txid
            );
        }

        let event = &res.events[2];
        if let BtcEvent::Update(TxUpdate {
            direction,
            txid,
            address,
            confirmations,
            ..
        }) = event
        {
            assert_eq!(*direction, TxDirection::Deposit);
            assert_eq!(txid.0, dep_txid);
            assert_eq!(address.0, deposit_address);
            assert_eq!(
                *confirmations, 0,
                "Expected confirmation counter is 0 after cancel"
            )
        } else {
            assert!(
                false,
                "Wrong type of event {:?}, expected deposit with txid {:?}",
                event, dep_txid
            );
        }
    })
    .await;
}

#[tokio::test]
async fn process_withdrawal_request(){
    run_test(|btc, api| async move{
        fund_wallet(&btc);
        let sk1bytes = [226, 143, 42, 33, 23, 231, 50, 229, 188, 25, 0, 63, 245, 176, 125, 158, 27, 252, 214, 95, 182, 243, 70, 176, 48, 9, 105, 34, 180, 198, 131, 6];
        let sk2bytes = [197, 103, 161, 120, 28, 231, 101, 35, 34, 117, 53, 115, 210, 176, 147, 227, 72, 177, 3, 11, 69, 147, 176, 246, 176, 171, 80, 1, 68, 143, 100, 96];
        let sk1 = p256::SecretKey::from_be_bytes(&sk1bytes).unwrap();
        let sk2 = p256::SecretKey::from_be_bytes(&sk2bytes).unwrap();
        let pk1 = sk1.public_key();
        let pk2 = sk2.public_key();
        let addr = new_address(&btc).to_string();
        let id = uuid::Uuid::new_v4();
        let user = "test_user".to_owned();
        let address = CurrencyAddress::BTC(hexstody_api::domain::BtcAddress{addr});
        let created_at = "now".to_owned();
        let amount = 10000000;
        let url = "http://127.0.0.1:8080".to_owned();
        let verdict = WithdrawalRequestDecisionType::Confirm;
        let msg = [
            json::to_string(&id).unwrap(), 
            user.clone(), 
            json::to_string(&address).unwrap(), 
            created_at.clone(), 
            amount.to_string(),
            url,
            json::to_string(&verdict).unwrap(), 
            ].join(":");
        let nonce1 = 0;
        let nonce2 = 1;
        let msg1 = [msg.clone(), nonce1.to_string()].join(":");
        let msg2 = [msg, nonce2.to_string()].join(":");
        println!("msg1  : {}",msg1);
        let sig1 = SigningKey::from(sk1).sign(msg1.as_bytes());
        let sig2 = SigningKey::from(sk2).sign(msg2.as_bytes());
        let ws1 = WithdrawalSignature {signature: sig1, public_key: pk1, nonce: nonce1, verdict};
        let ws2 = WithdrawalSignature {signature: sig2, public_key: pk2, nonce: nonce1, verdict};
        let confirmations = vec![ws1, ws2];
        let rejections = vec![];
        let cw = ConfirmedWithdrawal{id, user, address, created_at, amount, confirmations, rejections};
        let resp = api.withdraw_btc(cw).await;
        assert!(resp.is_ok(), "Failed to post tx");
    }).await;
}

// Withdraw unconfirmed transation
#[tokio::test]
async fn withdraw_unconfirmed_test() {
    run_test(|btc, api| async move {
        fund_wallet(&btc);
        let deposit_address = new_address(&btc);
        let dep_txid = send_funds(&btc, &deposit_address, Amount::from_sat(1000));
        let res = api.poll_events().await.expect("poll events");
        assert_eq!(res.events.len(), 2);
        let event = &res.events[0];
        if let BtcEvent::Update(TxUpdate {
            direction,
            txid,
            address,
            confirmations,
            ..
        }) = event
        {
            assert_eq!(*direction, TxDirection::Withdraw);
            assert_eq!(txid.0, dep_txid);
            assert_eq!(address.0, deposit_address);
            assert_eq!(*confirmations, 0);
        } else {
            assert!(
                false,
                "Wrong type of event {:?}, expected deposit with txid {:?}",
                event, dep_txid
            );
        }
    })
    .await;
}

// Withdraw confirmed transation
#[tokio::test]
async fn withdraw_confirmed_test() {
    run_test(|btc, api| async move {
        fund_wallet(&btc);
        let deposit_address = new_address(&btc);
        let dep_txid = send_funds(&btc, &deposit_address, Amount::from_sat(1000));
        mine_blocks(&btc, 1);
        let res = api.poll_events().await.expect("poll events");
        assert_eq!(res.events.len(), 2);
        let event = &res.events[0];
        if let BtcEvent::Update(TxUpdate {
            direction,
            txid,
            address,
            confirmations,
            ..
        }) = event
        {
            assert_eq!(*direction, TxDirection::Withdraw);
            assert_eq!(txid.0, dep_txid);
            assert_eq!(address.0, deposit_address);
            assert_eq!(*confirmations, 1);
        } else {
            assert!(
                false,
                "Wrong type of event {:?}, expected deposit with txid {:?}",
                event, dep_txid
            );
        }
    })
    .await;
}

// Test whether the confirmation of withdrawal is detected
#[tokio::test]
async fn withdraw_slow_confirmed_test() {
    run_test(|btc, api| async move {
        fund_wallet(&btc);
        let deposit_address = new_address(&btc);
        let dep_txid = send_funds(&btc, &deposit_address, Amount::from_sat(1000));
        let res = api.poll_events().await.expect("poll events");
        assert_eq!(res.events.len(), 2);

        mine_blocks(&btc, 1);
        let res = api.poll_events().await.expect("poll events");
        assert_eq!(res.events.len(), 2);
        let event = &res.events[0];
        if let BtcEvent::Update(TxUpdate {
            direction,
            txid,
            address,
            confirmations,
            ..
        }) = event
        {
            assert_eq!(*direction, TxDirection::Withdraw);
            assert_eq!(txid.0, dep_txid);
            assert_eq!(address.0, deposit_address);
            assert_eq!(*confirmations, 1);
        } else {
            assert!(
                false,
                "Wrong type of event {:?}, expected deposit with txid {:?}",
                event, dep_txid
            );
        }
    })
    .await;
}

// Test blocks after confirmed withdraw
#[tokio::test]
async fn withdraw_many_confirmed_test() {
    run_test(|btc, api| async move {
        fund_wallet(&btc);
        let deposit_address = new_address(&btc);
        let _ = send_funds(&btc, &deposit_address, Amount::from_sat(1000));
        let res = api.poll_events().await.expect("poll events");
        assert_eq!(res.events.len(), 2);

        mine_blocks(&btc, 1);
        let res = api.poll_events().await.expect("poll events");
        assert_eq!(res.events.len(), 2);

        let height = btc.get_block_count().expect("block count");
        mine_blocks(&btc, 1);
        let res = api.poll_events().await.expect("poll events");
        assert_eq!(res.events.len(), 0);
        assert_eq!(res.height, height + 1);
    })
    .await;
}

// Withdraw unconfirmed transation and cancel it
#[tokio::test]
async fn cancel_unconfirmed_withdraw_test() {
    run_two_nodes_test(|btc, other, api| async move {
        fund_wallet(&btc);
        let withdraw_address = new_address(&other);
        let dep_txid = send_funds(&btc, &withdraw_address, Amount::from_sat(1000));
        let res = api.poll_events().await.expect("Poll events");
        assert_eq!(res.events.len(), 1);

        let bumped_res = bumpfee(&btc, &dep_txid, None, None, None, None).expect("bump fee");
        let res = api.poll_events().await.expect("Poll events");
        assert_eq!(res.events.len(), 2, "Unexpected events: {:?}", res.events);

        mine_blocks(&btc, 1);
        let res = api.poll_events().await.expect("Poll events");
        assert_eq!(res.events.len(), 1, "Unexpected events: {:?}", res.events);

        let event = &res.events[0];
        if let BtcEvent::Update(TxUpdate {
            direction,
            txid,
            address,
            confirmations,
            conflicts,
            ..
        }) = event
        {
            assert_eq!(*direction, TxDirection::Withdraw);
            assert_eq!(txid.0, bumped_res.txid);
            assert_eq!(conflicts, &vec![BtcTxid(dep_txid)]);
            assert_eq!(address.0, withdraw_address);
            assert_eq!(*confirmations, 1);
        } else {
            assert!(
                false,
                "Wrong type of event {:?}, expected deposit with txid {:?}",
                event, dep_txid
            );
        }
    })
    .await;
}

// Withdraw confirmed transation and cancel it
#[tokio::test]
async fn cancel_confirmed_withdraw_test() {
    run_two_nodes_test(|btc, other, api| async move {
        fund_wallet(&btc);
        let withdraw_address = new_address(&other);
        let dep_txid = send_funds(&btc, &withdraw_address, Amount::from_sat(1000));
        mine_blocks(&btc, 1);
        let res = api.poll_events().await.expect("Poll events");
        assert_eq!(res.events.len(), 1);

        let event = &res.events[0];
        if let BtcEvent::Update(TxUpdate {
            direction,
            txid,
            address,
            confirmations,
            ..
        }) = event
        {
            assert_eq!(*direction, TxDirection::Withdraw);
            assert_eq!(txid.0, dep_txid);
            assert_eq!(address.0, withdraw_address);
            assert_eq!(*confirmations, 1, "Confirmed withdrawal");
        } else {
            assert!(
                false,
                "Wrong type of event {:?}, expected deposit with txid {:?}",
                event, dep_txid
            );
        }

        let last_block = btc.get_best_block_hash().expect("best block");
        btc.invalidate_block(&last_block).expect("forget block");

        let res = api.poll_events().await.expect("Poll events");
        assert_eq!(res.events.len(), 1, "Unexpected events: {:?}", res.events);

        let event = &res.events[0];
        if let BtcEvent::Update(TxUpdate {
            direction,
            txid,
            address,
            confirmations,
            ..
        }) = event
        {
            assert_eq!(*direction, TxDirection::Withdraw);
            assert_eq!(txid.0, dep_txid);
            assert_eq!(address.0, withdraw_address);
            assert_eq!(
                *confirmations, 0,
                "Expected confirmation counter is 0 after cancel"
            )
        } else {
            assert!(
                false,
                "Wrong type of event {:?}, expected deposit with txid {:?}",
                event, dep_txid
            );
        }
    })
    .await;
}

// Request fees from btc-node
#[tokio::test]
async fn get_fees_from_node_test() {
    run_test(|_, api| async move {
        let fee = api.get_fees().await.expect("Failed to get fee value");
        println!("{:?}", fee);
        assert_eq!(fee.fee_rate, 5, "Fee value is different than expected");
        assert!(fee.block.is_none(), "Block? How?");
    }).await;
}