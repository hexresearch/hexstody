use std::sync::Arc;

use hexstody_api::domain::{CurrencyAddress, EthAccount, BtcAddress, Erc20Token, Erc20};
use hexstody_api::{types::SignatureData, domain::Currency};
use hexstody_api::error;
use hexstody_btc_client::client::BtcClient;
use hexstody_db::update::{StateUpdate, UpdateBody};
use hexstody_eth_client::client::EthClient;
use hexstody_sig::{SignatureVerificationData, SignatureVerificationConfig};
use rocket::{serde::json, State};
use serde::Serialize;
use tokio::sync::{mpsc, Mutex};
use hexstody_db::state::State as DbState;
use log::*;

/// Guard operator handle from non-authorized user
pub fn guard_op_signature<T: Serialize>(
    config: &SignatureVerificationConfig,
    uri: String,
    signature_data: SignatureData,
    body: &T
) -> error::Result<()>{
    let url = [config.domain.clone(), uri].join("");
    let message = json::to_string(body).unwrap();
    let signature_verification_data = SignatureVerificationData {
        url,
        signature: signature_data.signature,
        nonce: signature_data.nonce,
        message: Some(message),
        public_key: signature_data.public_key,
    };
    signature_verification_data
        .verify(config.operator_public_keys.clone())
        .map_err(|e| error::Error::SignatureError(format!("{:?}", e)).into())
}

/// Guard operator handle from non-authorized user. Special case for when request has no body attached
pub fn guard_op_signature_nomsg(
    config: &SignatureVerificationConfig,
    uri: String,
    signature_data: SignatureData,
) -> error::Result<()>{
    let url = [config.domain.clone(), uri].join("");
    let signature_verification_data = SignatureVerificationData {
        url,
        signature: signature_data.signature,
        nonce: signature_data.nonce,
        message: None,
        public_key: signature_data.public_key,
    };
    signature_verification_data
        .verify(config.operator_public_keys.clone())
        .map_err(|e| error::Error::SignatureError(format!("{:?}", e)).into())
}

static HEXSTODY_EXCHANGE_USER: &str = "hexstody-exchange"; 

pub async fn get_deposit_address(
    btc_client: &State<BtcClient>,
    eth_client: &State<EthClient>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    state: &State<Arc<Mutex<DbState>>>,
    currency: Currency,
) -> Result<CurrencyAddress, error::Error> {
    match currency {
        Currency::BTC => allocate_address(btc_client, eth_client, updater, currency).await,
        Currency::ETH | Currency::ERC20(_) => {
            let db_state = state.lock().await;
            let deposit_addresses: Vec<CurrencyAddress> = db_state
                .users
                .get(HEXSTODY_EXCHANGE_USER)
                .ok_or(error::Error::NoUserFound)?
                .currencies
                .get(&currency)
                .ok_or(error::Error::NoUserCurrency(currency.clone()))?
                .deposit_info
                .clone();
            if deposit_addresses.is_empty() {
                allocate_address(btc_client, eth_client, updater, currency.clone()).await
            } else {
                Ok(deposit_addresses[0].clone())
            }
        }
    }
}

async fn allocate_address(
    btc_client: &State<BtcClient>,
    eth_client: &State<EthClient>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    currency: Currency,
) -> Result<CurrencyAddress, error::Error> {
    match currency {
        Currency::BTC => allocate_btc_address(btc_client, updater).await,
        Currency::ETH => allocate_eth_address(eth_client, updater).await,
        Currency::ERC20(token) => allocate_erc20_address(eth_client, updater, token).await,
    }
}

async fn allocate_btc_address(
    btc: &State<BtcClient>,
    updater: &State<mpsc::Sender<StateUpdate>>,
) -> Result<CurrencyAddress, error::Error> {
    let address = btc.deposit_address().await.map_err(|e| {
        error!("{}", e);
        error::Error::FailedGenAddress(Currency::BTC)
    })?;
    let packed_address = CurrencyAddress::BTC(BtcAddress{addr: format!("{}", address)});
    updater.send(StateUpdate::new(UpdateBody::ExchangeAddress(packed_address.clone())))
        .await.map_err(|e| error::Error::GenericError(e.to_string()))?;
    Ok(packed_address)
}

async fn allocate_eth_address(
    eth_client: &State<EthClient>,
    updater: &State<mpsc::Sender<StateUpdate>>,
) -> Result<CurrencyAddress, error::Error> {
    let user_data = eth_client
        .get_user_data(HEXSTODY_EXCHANGE_USER)
        .await
        .map_err(|e| error::Error::FailedETHConnection(e.to_string()))?;
    let packed_address = CurrencyAddress::ETH(EthAccount {
        account: user_data.address,
    });
    updater.send(StateUpdate::new(UpdateBody::ExchangeAddress(packed_address.clone())))
        .await.map_err(|e| error::Error::GenericError(e.to_string()))?;
    Ok(packed_address)
}

async fn allocate_erc20_address(
    eth_client: &State<EthClient>,
    updater: &State<mpsc::Sender<StateUpdate>>,
    token: Erc20Token,
) -> Result<CurrencyAddress, error::Error> {
    let user_data = eth_client
        .get_user_data(HEXSTODY_EXCHANGE_USER)
        .await
        .map_err(|e| error::Error::FailedETHConnection(e.to_string()))?;
    let packed_address = CurrencyAddress::ERC20(Erc20 {
        token: token,
        account: EthAccount {
            account: user_data.address,
        },
    });
    updater.send(StateUpdate::new(UpdateBody::ExchangeAddress(packed_address.clone())))
        .await.map_err(|e| error::Error::GenericError(e.to_string()))?;
    Ok(packed_address)
}