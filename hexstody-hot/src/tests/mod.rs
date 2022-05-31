mod runner;

use bitcoin::{Address, Amount};
use hexstody_api::{domain::*, types::*};
use hexstody_btc_test::helpers::*;
use runner::*;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_simple() {
    run_test(|env| async move {
        env.hot_client.ping().await.expect("Ping finished");
    })
    .await;
}

#[tokio::test]
async fn test_auth_email() {
    run_test(|env| async move {
        let user = "aboba@mail.com".to_owned();
        let password = "123456".to_owned();

        let res = env
            .hot_client
            .signin_email(SigninEmail {
                user: user.clone(),
                password: password.clone(),
            })
            .await;
        assert!(!res.is_ok());

        env.hot_client
            .signup_email(SignupEmail {
                user: user.clone(),
                password: password.clone(),
            })
            .await
            .expect("Signup");

        let res = env.hot_client.logout().await;
        assert!(!res.is_ok(), "Logout before signing");

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
        assert!(!res.is_ok(), "Double logout");
    })
    .await;
}

#[tokio::test]
async fn test_btc_deposit() {
    run_with_user(|env| async move {
        fund_wallet(&env.btc_node);

        let dep_info = env
            .hot_client
            .get_deposit(Currency::BTC)
            .await
            .expect("Deposit address");

        let dep_address = Address::from_str(&dep_info.address).expect("Bitcoin address");
        let amount = Amount::from_sat(10_000);
        send_funds(&env.btc_node, &dep_address, amount);
        mine_blocks(&env.btc_node, 1);
        sleep(Duration::from_millis(500)).await;

        let balances = env.hot_client.get_balance().await.expect("Balances");
        assert_eq!(
            Some(amount),
            balances
                .by_currency(&Currency::BTC)
                .map(|i| Amount::from_sat(i.value.try_into().expect("Positive balance")))
        );
    })
    .await;
}

#[tokio::test]
async fn test_btc_unconfirmed_deposit() {
    run_with_user(|env| async move {
        fund_wallet(&env.btc_node);

        let dep_info = env
            .hot_client
            .get_deposit(Currency::BTC)
            .await
            .expect("Deposit address");

        let dep_address = Address::from_str(&dep_info.address).expect("Bitcoin address");
        let amount = Amount::from_sat(10_000);
        send_funds(&env.btc_node, &dep_address, amount);
        // mine_blocks(&env.btc_node, 1);
        sleep(Duration::from_millis(500)).await;

        let balances = env.hot_client.get_balance().await.expect("Balances");
        assert_eq!(
            Some(amount),
            balances
                .by_currency(&Currency::BTC)
                .map(|i| Amount::from_sat(i.value.try_into().expect("Positive balance")))
        );
    })
    .await;
}

#[tokio::test]
async fn test_btc_rbf_deposit() {
    run_with_user(|env| async move {
        fund_wallet(&env.btc_node);

        let dep_info = env
            .hot_client
            .get_deposit(Currency::BTC)
            .await
            .expect("Deposit address");

        let dep_address = Address::from_str(&dep_info.address).expect("Bitcoin address");
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
                .map(|i| Amount::from_sat(i.value.try_into().expect("Positive balance")))
        );
    })
    .await;
}
