use chrono::Utc;
use hexstody_api::{
    domain::{BTCTxid, CurrencyTxId},
    types::{ConfirmedWithdrawal, LimitSpan},
    domain::currency::CurrencyAddress
};
use hexstody_btc_api::events::*;
use hexstody_btc_client::client::BtcClient;
use hexstody_eth_client::client::EthClient;
use hexstody_db::{
    state::State,
    update::{
        btc::BestBtcBlock,
        results::UpdateResult,
        withdrawal::{WithdrawCompleteInfo, WithdrawalRejectInfo},
        StateUpdate, UpdateBody,
    },
};
use log::*;
use std::{sync::Arc, vec};
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep;
use tokio_cron_scheduler::*;

pub async fn update_results_worker(
    btc_client: BtcClient,
    eth_client: EthClient,
    state_mx: Arc<Mutex<State>>,
    mut update_receiver: mpsc::Receiver<UpdateResult>,
    update_sender: mpsc::Sender<StateUpdate>,
) {
    trace!("Starting update results worker");
    loop {
        match update_receiver.recv().await {
            Some(upd) => match upd {
                UpdateResult::WithdrawConfirmed(id) => {
                    let sreq = {
                        let state = state_mx.lock().await;
                        state.get_withdrawal_request(id)
                    };
                    if let Some(req) = sreq {
                        let confirmations = req
                            .confirmations
                            .iter()
                            .map(|wrd| wrd.clone().into())
                            .collect();
                        let rejections = req
                            .rejections
                            .iter()
                            .map(|wrd| wrd.clone().into())
                            .collect();
                        let cw = ConfirmedWithdrawal {
                            id,
                            user: req.user.clone(),
                            address: req.address.clone(),
                            created_at: req.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                            amount: req.amount,
                            confirmations,
                            rejections,
                        };
                        info!("=================DEBUG=================");
                        info!("===============<UPDATE>================");
                        info!("{:}",&id);
                        info!("{:}",req.user);
                        match req.address.clone() {
                            CurrencyAddress::BTC(a) => {
                                info!("===============<BTC>================");
                                info!("{:}",a);
                                match btc_client.withdraw_btc(cw).await {
                                    Ok(resp) => {
                                        debug!("withdraw_btc_resp: {:?}", resp);
                                        let txid = resp.txid.0.to_string();
                                        let bod =
                                            UpdateBody::WithdrawalRequestComplete(WithdrawCompleteInfo {
                                                id: resp.id,
                                                confirmed_at: Utc::now().naive_utc(),
                                                txid: CurrencyTxId::BTC(BTCTxid { txid }),
                                                fee: resp.fee,
                                                input_addresses: resp.input_addresses,
                                                output_addresses: resp.output_addresses,
                                                request_type: hexstody_db::state::WithdrawalRequestType::OverLimit,
                                            });
                                        if let Err(e) = update_sender.send(StateUpdate::new(bod)).await {
                                            debug!("Failed to send update with confirmation: {}", e);
                                        };
                                    }
                                    Err(e) => {
                                        debug!("Failed to post tx: {:?}", e);
                                        let info = WithdrawalRejectInfo {
                                            id,
                                            reason: format!("{}", e),
                                        };
                                        let bod = UpdateBody::WithdrawalRequestNodeRejected(info);
                                        if let Err(e) = update_sender.send(StateUpdate::new(bod)).await {
                                            debug!("Failed to send update with node rejection: {}", e);
                                        };
                                    }
                                }
                                info!("===============</BTC>================");
                            },
                            CurrencyAddress::ETH(a) => {
                                info!("===============<ETH>================");
                                match eth_client.send_tx(&req.user.clone(),&a.account,&cw.amount.to_string()).await {
                                    Ok(resp) => {
                                        info!("withdraw_eth_resp: {:?}", resp);
                                    }
                                    Err(e) => {
                                        debug!("Failed to post tx: {:?}", e);
                                        let info = WithdrawalRejectInfo {
                                            id,
                                            reason: format!("{}", e),
                                        };
                                        let bod = UpdateBody::WithdrawalRequestNodeRejected(info);
                                        if let Err(e) = update_sender.send(StateUpdate::new(bod)).await {
                                            debug!("Failed to send update with node rejection: {}", e);
                                        };
                                    }
                                }
                                info!("===============</ETH>================");
                            },
                            CurrencyAddress::ERC20(a) => {
                                info!("===============<ERC20>================");
                                info!("{:}",a.token.contract.clone());
                                match eth_client.send_tx_erc20(&req.user.clone(),&a.account.account,&a.token.contract,&cw.amount.to_string()).await {
                                    Ok(resp) => {
                                        info!("withdraw_eth_resp: {:?}", resp);
                                    }
                                    Err(e) => {
                                        debug!("Failed to post tx: {:?}", e);
                                        let info = WithdrawalRejectInfo {
                                            id,
                                            reason: format!("{}", e),
                                        };
                                        let bod = UpdateBody::WithdrawalRequestNodeRejected(info);
                                        if let Err(e) = update_sender.send(StateUpdate::new(bod)).await {
                                            debug!("Failed to send update with node rejection: {}", e);
                                        };
                                    }
                                }
                                info!("===============</ERC20>================");
                            },
                        }
                        info!("===============</UPDATE>================");
                        info!("=================DEBUG=================");
                    }
                }
                UpdateResult::WithdrawalUnderlimit(wr) => {
                    let cw = ConfirmedWithdrawal{
                        id: wr.id,
                        user: wr.user,
                        address: wr.address,
                        created_at: wr.created_at.to_string(),
                        amount: wr.amount,
                        confirmations: vec![],
                        rejections: vec![],
                    };
                    match btc_client.withdraw_under_limit(cw).await {
                        Ok(resp) => {
                            debug!("withdraw_btc_resp: {:?}", resp);
                            let txid = resp.txid.0.to_string();
                            let bod =
                                UpdateBody::WithdrawalRequestComplete(WithdrawCompleteInfo {
                                    id: resp.id,
                                    confirmed_at: Utc::now().naive_utc(),
                                    txid: CurrencyTxId::BTC(BTCTxid { txid }),
                                    fee: resp.fee,
                                    input_addresses: resp.input_addresses,
                                    output_addresses: resp.output_addresses,
                                    request_type: hexstody_db::state::WithdrawalRequestType::UnderLimit,
                                });
                            if let Err(e) = update_sender.send(StateUpdate::new(bod)).await {
                                debug!("Failed to send update with confirmation: {}", e);
                            };
                        }
                        Err(e) => {
                            debug!("Failed to post tx: {:?}", e);
                            let info = WithdrawalRejectInfo {
                                id: wr.id,
                                reason: format!("{}", e),
                            };
                            let bod = UpdateBody::WithdrawalRequestNodeRejected(info);
                            if let Err(e) = update_sender.send(StateUpdate::new(bod)).await {
                                debug!("Failed to send update with node rejection: {}", e);
                            };
                        }
                    }
                },
            },
            None => break,
        }
    }
}

pub async fn btc_worker(
    btc_client: BtcClient,
    state_mx: Arc<Mutex<State>>,
    update_sender: mpsc::Sender<StateUpdate>,
) {
    trace!("Starting BTC worker");
    loop {
        match btc_client.poll_events().await {
            Ok(events) => {
                process_btc_events(state_mx.clone(), &update_sender, events).await;
            }
            Err(e) => {
                error!("BTC module error: {e}");
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

pub async fn process_btc_events(
    state_mx: Arc<Mutex<State>>,
    update_sender: &mpsc::Sender<StateUpdate>,
    events: BtcEvents,
) {
    let block_hash = events.hash.0.to_string();
    {
        let state = state_mx.lock().await;
        if state.btc_state.block_hash != block_hash {
            update_sender
                .send(StateUpdate::new(UpdateBody::BestBtcBlock(BestBtcBlock {
                    height: events.height,
                    block_hash,
                })))
                .await
                .unwrap();
        }

        for event in events.events {
            match event {
                BtcEvent::Update(upd) => {
                    if upd.direction == TxDirection::Deposit {
                        update_sender
                            .send(StateUpdate::new(UpdateBody::UpdateBtcTx(upd.into())))
                            .await
                            .unwrap();
                    }
                }
                BtcEvent::Cancel(cnl) => {
                    update_sender
                        .send(StateUpdate::new(UpdateBody::CancelBtcTx(cnl.into())))
                        .await
                        .unwrap();
                }
            }
        }
    }
}

pub async fn cron_workers(
    update_sender: mpsc::Sender<StateUpdate>,
) {
    info!("Starting cleanup worker");
    let mut sched = JobScheduler::new().await.expect("Failed to create scheduler");

    sched.shutdown_on_ctrl_c();
    sched.set_shutdown_handler(Box::new(|| {
        Box::pin(async move {
          println!("Shut down done");
        })
      }));

    let send_daily = update_sender.clone();
    let send_weekly = update_sender.clone();

    let _daily_id = sched.add(Job::new_async("0 0 0 * * *", move |_, _| {
        let update_sender = send_daily.clone();
        let upd = StateUpdate::new(UpdateBody::ClearLimits(LimitSpan::Day));
        Box::pin(async move {
            info!("Starting daily cleanup");
            if let Err(e) = update_sender.send(upd.clone()).await{
                error!("{:?}", e);
            }
        })
    }).expect("Failed to create daily job"));

    let _weekly_id = sched.add(Job::new_async("0 0 0 * * Mon *", move |_, _| {
        let update_sender = send_weekly.clone();
        let upd = StateUpdate::new(UpdateBody::ClearLimits(LimitSpan::Week));
        Box::pin(async move {
            info!("Starting weekly cleanup");
            if let Err(e) = update_sender.send(upd.clone()).await{
                error!("{:?}", e);
            }
        })
    }).expect("Failed to create weekly job"));

    let _monthly_id = sched.add(Job::new_async("0 0 0 1 * * *", move |_, _| {
        let update_sender = update_sender.clone();
        let upd = StateUpdate::new(UpdateBody::ClearLimits(LimitSpan::Month));
        Box::pin(async move {
            info!("Starting monthly cleanup");
            if let Err(e) = update_sender.send(upd.clone()).await{
                error!("{:?}", e);
            }
        })
    }).expect("Failed to create monthly job"));

    sched.start().await.expect("Some error in scheduling");
}
