mod runner;

use bitcoin::{Address, Amount};
use hexstody_api::{domain::*, types::*};
use hexstody_btc_test::helpers::*;
use uuid::Uuid;
use runner::*;
use serial_test::serial;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
#[serial]
async fn test_simple() {
    run_test(|env| async move {
        env.hot_client.ping().await.expect("Ping finished");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_auth_email() {
    run_test(|env| async move {
        let user = "aboba@mail.com".to_owned();
        let password = "123456".to_owned();
        let _removed = env.hot_client.test_only_remove_eth_user(&user).await;
        let res = env
            .hot_client
            .signin_email(SigninEmail {
                user: user.clone(),
                password: password.clone(),
            })
            .await;
        assert!(!res.is_ok());

        let wrong_invite = Invite{invite: Uuid::new_v4()};
        let res = env.hot_client
            .signup_email(SignupEmail {
                user: user.clone(),
                invite: wrong_invite,
                password: password.clone(),
            })
            .await;
        assert!(res.is_err(), "Signed up with a wrong invite!");
 
        let sk = env.secret_keys.get(0).unwrap();
        let invite_req = InviteRequest{ label: "test invite".to_string() };
        let invite_resp = env.hot_client.gen_invite(sk.clone(), invite_req).await.expect("Failed to register invite");
        env.hot_client
            .signup_email(SignupEmail {
                user: user.clone(),
                invite: invite_resp.invite,
                password: password.clone(),
            })
            .await
            .expect("Signup");

        let res = env.hot_client.logout().await;
        // switched from !res to res.is_ok() since with redirects double logouts return Ok(Redirect) to /
        assert!(res.is_ok(), "Logout before signing");
        let res = env
            .hot_client
            .signin_email(SigninEmail {
                user: user.clone(),
                password: "wrong".to_owned(),
            })
            .await;
        assert!(!res.is_ok(), "Wrong password passes");
        env.hot_client
            .signin_email(SigninEmail {
                user: user.clone(),
                password: password.clone(),
            })
            .await
            .expect("Signin");
        env.hot_client.logout().await.expect("Logout");
        let res = env.hot_client.logout().await;
        // same as with logout before signing
        assert!(res.is_ok(), "Double logout");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_btc_deposit() {
    run_with_user(|env| async move {
        fund_wallet(&env.btc_node);

        let dep_info = env
            .hot_client
            .get_deposit_address(Currency::BTC)
            .await
            .expect("Deposit address");

        let dep_address = Address::from_str(&dep_info.address()).expect("Bitcoin address");
        let amount = Amount::from_sat(10_000);
        send_funds(&env.btc_node, &dep_address, amount);
        mine_blocks(&env.btc_node, 1);
        sleep(Duration::from_millis(500)).await;

        let balances = env.hot_client.get_balance().await.expect("Balances");
        assert_eq!(
            Some(amount),
            balances
                .by_currency(&Currency::BTC)
                .map(|i| Amount::from_sat(i.value.amount.try_into().expect("Positive balance")))
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_btc_unconfirmed_deposit() {
    run_with_user(|env| async move {
        fund_wallet(&env.btc_node);

        let dep_info = env
            .hot_client
            .get_deposit_address(Currency::BTC)
            .await
            .expect("Deposit address");

        let dep_address = Address::from_str(&dep_info.address()).expect("Bitcoin address");
        let amount = Amount::from_sat(10_000);
        send_funds(&env.btc_node, &dep_address, amount);
        // mine_blocks(&env.btc_node, 1);
        sleep(Duration::from_millis(500)).await;

        let balances = env.hot_client.get_balance().await.expect("Balances");
        assert_eq!(
            Some(amount),
            balances
                .by_currency(&Currency::BTC)
                .map(|i| Amount::from_sat(i.value.amount.try_into().expect("Positive balance")))
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_btc_rbf_0conf_deposit() {
    run_with_user(|env| async move {
        fund_wallet(&env.btc_node);

        let dep_info = env
            .hot_client
            .get_deposit_address(Currency::BTC)
            .await
            .expect("Deposit address");

        let dep_address = Address::from_str(&dep_info.address()).expect("Bitcoin address");
        let amount = Amount::from_sat(10_000);
        let dep_txid = send_funds(&env.btc_node, &dep_address, amount);

        sleep(Duration::from_millis(1000)).await;

        let _ = bumpfee(&env.btc_node, &dep_txid, None, None, None, None).expect("bump fee");

        sleep(Duration::from_millis(1000)).await;

        let balances = env.hot_client.get_balance().await.expect("Balances");
        assert_eq!(
            Some(Amount::from_sat(0)),
            balances
                .by_currency(&Currency::BTC)
                .map(|i| Amount::from_sat(i.value.amount.try_into().expect("Positive balance")))
        );
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_btc_rbf_1conf_deposit() {
    run_with_user(|env| async move {
        fund_wallet(&env.btc_node);

        let dep_info = env
            .hot_client
            .get_deposit_address(Currency::BTC)
            .await
            .expect("Deposit address");

        let dep_address = Address::from_str(&dep_info.address()).expect("Bitcoin address");
        let amount = Amount::from_sat(10_000);
        let dep_txid = send_funds(&env.btc_node, &dep_address, amount);

        sleep(Duration::from_millis(1000)).await;

        let _ = bumpfee(&env.btc_node, &dep_txid, None, None, None, None).expect("bump fee");

        sleep(Duration::from_millis(1000)).await;
        mine_blocks(&env.btc_node, 1);
        sleep(Duration::from_millis(1000)).await;

        let balances = env.hot_client.get_balance().await.expect("Balances");
        assert_eq!(
            Some(Amount::from_sat(10_000)),
            balances
                .by_currency(&Currency::BTC)
                .map(|i| Amount::from_sat(i.value.amount.try_into().expect("Positive balance")))
        );
    })
    .await;
}