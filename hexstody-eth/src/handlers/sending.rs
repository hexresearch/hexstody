use crate::types::*;
use crate::utils::*;
use crate::node_calls;
use crate::db_functions;
use crate::conf::{NodeConfig, load_config};

use rocket::{
    get,post,State,
    http::{Status, ContentType},
    serde::json::Json
};

use rocket_db_pools::{Database, Connection};
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};

use web3::{
    contract::{Contract, Options},
    ethabi::ethereum_types::U256,
    types::{Address, TransactionParameters,TransactionRequest, H160},
    api::{Web3Api}
};

use std::str::FromStr;
use secp256k1::SecretKey;


#[openapi(tag = "send")]
#[get("/send/eth/login/<login>/<recipient>/<volume>")]
pub async fn send_eth_from_login(login: &str,
                  recipient: &str,
                  volume: &str,
                  cfg: &State<NodeConfig>,
                  db: Connection<MyDb>) -> (Status, (ContentType, String)) {
  let user = db_functions::pg_query_user(db,login).await.unwrap();
  let user_data : UserData = serde_json::from_value(user.data.unwrap()).unwrap();
  let user_address = &user.address;
  let user_eth_bal = &user_data.balanceEth;
  log::warn!("=======================<SENDING>==========================");
  log::warn!("Username {:?}",login);
  log::warn!("Address {:?}",user_address);
  log::warn!("Balance {:?}",user_eth_bal);
  log::warn!("Parsing {:?}",&volume);
  let eth_tx = EthTransactionNoData {
      from : user_address.to_string(),
      to : recipient.to_string(),
      value: to_hex_str(volume),
  };
  let r_send_tx = node_calls::send_eth_tx(cfg,&eth_tx);
  match r_send_tx{
      Err(e) => {
          log::warn!("NodeErr Response {:?}",&e);
          return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
      },
      Ok(send_tx) => {
          let json_res = serde_json::to_string(&send_tx);
          return (Status::Ok, (ContentType::JSON, json_res.unwrap()));
      }
  }
}

#[openapi(tag = "send")]
#[get("/send/eth/address/<address>/<recipient>/<volume>")]
pub async fn send_eth_from_address(address: &str,
                  recipient: &str,
                  volume: &str,
                  cfg: &State<NodeConfig>,
                  db: Connection<MyDb>) -> (Status, (ContentType, String)) {
  log::warn!("=======================<SENDING>==========================");
  log::warn!("Address {:?}",address);
  log::warn!("Recipient {:?}",recipient);
  log::warn!("Parsing {:?}",&volume);
  log::warn!("Hex {:?}",to_hex_str(volume));
  let eth_tx = EthTransactionNoData {
      from : address.to_string(),
      to : recipient.to_string(),
      value: to_hex_str(volume),
  };
  let r_send_tx = node_calls::send_eth_tx(cfg,&eth_tx);
  match r_send_tx{
      Err(e) => {
          log::warn!("NodeErr Response {:?}",&e);
          return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
      },
      Ok(send_tx) => {
          let json_res = serde_json::to_string(&send_tx);
          return (Status::Ok, (ContentType::JSON, json_res.unwrap()));
      }
  }
}

#[openapi(tag = "send")]
#[get("/unlock/<login>")]
pub async fn unlock_eth(login: &str,
                  cfg: &State<NodeConfig>,
                  db: Connection<MyDb>) -> (Status, (ContentType, String)) {
  let user = db_functions::pg_query_user(db,login).await.unwrap();
  let _user_data : UserData = serde_json::from_value(user.data.unwrap()).unwrap();
  let user_address = &user.address;
  let acc_per = AccountPersonal {
      address   : user_address.to_string(),
      passphrase : login.to_string(),
      duration   : 60,
  };
  let r_unlock_acc = node_calls::unlock_account(cfg,&acc_per);
  match r_unlock_acc{
      Err(e) => {
          log::warn!("NodeErr Response {:?}",&e);
          return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
      },
      Ok(unlock_acc) => {
          let json_res = serde_json::to_string(&unlock_acc);
          return (Status::Ok, (ContentType::JSON, json_res.unwrap()));
      }
  }
}


#[openapi(tag = "send")]
#[get("/senderc20tx/address/<address>/<token_address>/<recipient>/<volume>")]
pub async fn send_erc20_from_address(address: &str,
                  token_address: &str,
                  recipient: &str,
                  volume: &str,
                  cfg: &State<NodeConfig>,
                  db: Connection<MyDb>) -> (Status, (ContentType, String)) {
  log::warn!("=======================<SENDING ERC20>==========================");
  log::warn!("TokenAddress {:?}",token_address);
  log::warn!("Parsing {:?}",&volume);
//  let websocket = web3::transports::WebSocket::new(&"ws://localhost:8545".to_string()).await.unwrap();
//  let web3 = web3::Web3::new(websocket);
  let nurl: &str = &(cfg.nodeurl.clone() + ":" + &cfg.nodeport.to_string());
  let transport = web3::transports::Http::new(nurl).unwrap();
  let web3 = web3::Web3::new(transport);
  let h160addr = H160::from_str(address).unwrap();
  let h160recipient = H160::from_str(recipient).unwrap();
  let user_address: Address = address.to_string().parse().unwrap();
  let recipient_address: Address = recipient.to_string().parse().unwrap();
  let hex_volume = &to_hex_str_clean(&volume);
  let raw_hex_volume_command = "0000000000000000000000000000000000000000000000000000000000000000";
  let tx_data: String = "0xa9059cbb000000000000000000000000".to_string() +
        &recipient[2..address.len()] +
        &raw_hex_volume_command[0..(&raw_hex_volume_command.len()-hex_volume.len())] +
        hex_volume;
  log::warn!("Tx Data:\n {:?}",&tx_data);

  let eth_tx = EthTransactionDefaultGas {
      from : address.to_string(),
      to : token_address.to_string(),
      value: to_hex_str("0"),
      data: tx_data,
  };
  let r_send_tx = node_calls::send_erc20_tx(cfg,&eth_tx);

  match r_send_tx{
      Err(e) => {
          log::warn!("NodeErr Response {:?}",&e);
          return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
      },
      Ok(send_tx) => {
          let json_res = serde_json::to_string(&send_tx);
          return (Status::Ok, (ContentType::JSON, json_res.unwrap()));
      }
  }
}


#[openapi(skip)]
#[get("/senderc20tx/<login>/<taddr>/<recipient>/<volume>")]
pub async fn send_erc20(login: &str,
                  taddr: &str,
                  recipient: &str,
                  volume: &str,
                  cfg: &State<NodeConfig>,
                  db: Connection<MyDb>) -> (Status, (ContentType, String)) {
  let user = db_functions::pg_query_user(db,login).await.unwrap();
  let user_data : UserData = serde_json::from_value(user.data.unwrap()).unwrap();
  let user_address = &user.address;
  let user_eth_bal = &user_data.balanceEth;
  log::warn!("=======================<SENDING ERC20>==========================");
  log::warn!("Username {:?}",login);
  log::warn!("Address {:?}",user_address);
  log::warn!("TokenAddress {:?}",taddr);
  log::warn!("Balance {:?}",user_eth_bal);
  log::warn!("Parsing {:?}",&volume);
  let nurl: &str = &(cfg.nodeurl.clone() + ":" + &cfg.nodeport.to_string());
  let transport = web3::transports::Http::new(nurl).unwrap();
  let web3 = web3::Web3::new(transport);
  let h160taddr = H160::from_str(taddr).unwrap();
  let taddress: Address = taddr.to_string().parse().unwrap();
  let eth_tx = EthTransactionNoData {
      from : user_address.to_string(),
      to : recipient.to_string(),
      value: to_hex_str(volume),
  };
  let r_send_tx = node_calls::send_eth_tx(cfg,&eth_tx);
  match r_send_tx{
      Err(e) => {
          log::warn!("NodeErr Response {:?}",&e);
          return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
      },
      Ok(send_tx) => {
          let json_res = serde_json::to_string(&send_tx);
          return (Status::Ok, (ContentType::JSON, json_res.unwrap()));
      }
  }
}

#[openapi(tag = "signsend")]
#[get("/signingsend/eth/login/<login>/<recipient>/<volume>")]
pub async fn signsend(login: &str,
                  recipient: &str,
                  volume: &str,
                  cfg: &State<NodeConfig>,
                  db: Connection<MyDb>) -> (Status, (ContentType, String)) {
    let user = db_functions::pg_query_user(db,login).await.unwrap();
    let user_data : UserData = serde_json::from_value(user.data.unwrap()).unwrap();
    let user_address = &user.address;
    let user_eth_bal = &user_data.balanceEth;
    let h160addr = H160::from_str(user_address).unwrap();
    let h160recipient = H160::from_str(recipient).unwrap();
    log::warn!("=======================<SIGNEDSENDING>==========================");
    log::warn!("Username {:?}",login);
    log::warn!("Address {:?}",user_address);
    log::warn!("AddressH160 {:?}",h160addr);
    log::warn!("RecipientH160 {:?}",h160recipient);
    log::warn!("Balance {:?}",user_eth_bal);
    let nurl: &str = &(cfg.nodeurl.clone() + ":" + &cfg.nodeport.to_string());
    let transport = web3::transports::Http::new(nurl).unwrap();
    let web3 = web3::Web3::new(transport);

    let transport_i = web3::transports::Http::new("https://mainnet.infura.io/v3/e2f19e2685c64874b4495b3f37a10aa3").unwrap();
    let web3_i = web3::Web3::new(transport_i);

    let to = Address::from_str(recipient).unwrap();

//    let vol = volume.parse::<f64>().unwrap();
//    let vals = ((1000000000000000000.0*vol) as u64).to_string();
    let val_u= U256::from_dec_str(volume).unwrap();
    log::warn!("Sending value {:?}",val_u);

    let count = web3_i.eth().transaction_count(h160addr, None).await.unwrap();

    log::warn!("nonce count{:?}",count.to_string());

    let tx_object = TransactionRequest {
        from: h160addr,
        to: Some(to),
        value: Some(val_u),
        gas:Some(U256::from_dec_str("21000").unwrap()),
        gas_price:Some(U256::from_dec_str("45000000000").unwrap()),
        nonce:Some(count),
        ..Default::default()
    };

    let tx_object_clone = tx_object.clone();

    let r_signed = web3.personal().sign_transaction(tx_object, login).await;
    match r_signed{
        Err(e) => {
            log::warn!("Node Signing Error Response {:?}",&e);
            return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
        },
        Ok(raw_transaction) => {
            log::warn!("Signed raw transaction {:?}",raw_transaction);
            let result = web3_i.eth().send_raw_transaction(raw_transaction.raw).await;
            match result{
                Err(e) => {
                    log::warn!("Node Raw Sending Response {:?}",&e);
                    return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
                },
                Ok(tx_hash) => {
                    log::warn!("to: {:?}, val: {:?}, hash: {:?}",tx_object_clone.to, tx_object_clone.value, result);
                    let tx_data = SendTxData {
                        to:to.to_string(),
                        val:val_u.to_string(),
                        hash:tx_hash.to_string()
                    };
                    let json_res = serde_json::to_string(&tx_data);
                    return (Status::Ok, (ContentType::JSON, json_res.unwrap()));
                }
            }
        }
    }
}


#[openapi(tag = "signsend")]
#[get("/signingsend/erc20/login/<login>/<recipient>/<token_address>/<volume>")]
pub async fn signsend_erc20(login: &str,
                  recipient: &str,
                  token_address: &str,
                  volume: &str,
                  cfg: &State<NodeConfig>,
                  db: Connection<MyDb>) -> (Status, (ContentType, String)) {
    let user = db_functions::pg_query_user(db,login).await.unwrap();
    let user_data : UserData = serde_json::from_value(user.data.unwrap()).unwrap();
    let user_address = &user.address;
    let user_eth_bal = &user_data.balanceEth;
    let h160addr = H160::from_str(user_address).unwrap();
    let h160recipient = H160::from_str(recipient).unwrap();
    let h160token_address = H160::from_str(token_address).unwrap();
    let hex_volume = &to_hex_str_clean(&volume);
    let raw_hex_volume_command = "0000000000000000000000000000000000000000000000000000000000000000";
    let tx_data: String = "0xa9059cbb000000000000000000000000".to_string() +
          &recipient[2..recipient.len()] +
          &raw_hex_volume_command[0..(&raw_hex_volume_command.len()-hex_volume.len())] +
          hex_volume;

    let tx_data_bytes : web3::types::Bytes = web3::types::Bytes(tx_data.clone().into_bytes());
    log::warn!("=======================<SIGNEDSENDINGERC20>==========================");
    log::warn!("Username {:?}",login);
    log::warn!("Address {:?}",user_address);
    log::warn!("AddressH160 {:?}",h160addr);
    log::warn!("RecipientH160 {:?}",h160recipient);
    log::warn!("Token Address H160 {:?}",h160token_address);
    log::warn!("Balance {:?}",user_eth_bal);
    log::warn!("Tx Data:\n {:?}",tx_data.clone());

    let nurl: &str = &(cfg.nodeurl.clone() + ":" + &cfg.nodeport.to_string());
    let transport = web3::transports::Http::new(nurl).unwrap();
    let web3 = web3::Web3::new(transport);

    let transport_i = web3::transports::Http::new(nurl).unwrap();//web3::transports::Http::new("https://mainnet.infura.io/v3/e2f19e2685c64874b4495b3f37a10aa3").unwrap();
    let web3_i = web3::Web3::new(transport_i);
    let to = Address::from_str(token_address).unwrap();

//    let vol = volume.parse::<f64>().unwrap();
//    let vals = ((1000000000000000000.0*vol) as u64).to_string();
    let val_u= U256::from_dec_str(volume).unwrap();
    log::warn!("Sending value {:?}",val_u);

    let count = web3_i.eth().transaction_count(h160addr, None).await.unwrap();

    log::warn!("nonce count{:?}",count.to_string());

    let tx_object = TransactionRequest {
        from: h160addr,
        to: Some(to),
        value: Some(U256::from_dec_str("0").unwrap()),
        gas:Some(U256::from_dec_str("520000").unwrap()),
        gas_price:Some(U256::from_dec_str("1800000000").unwrap()),
        data:Some(tx_data_bytes.clone()),
        nonce:Some(count),
        ..Default::default()
    };

    let tx_object_clone = tx_object.clone();

    let r_signed = web3.personal().sign_transaction(tx_object, login).await;
    match r_signed{
        Err(e) => {
            log::warn!("Node Signing Error Response {:?}",&e);
            return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
        },
        Ok(raw_transaction) => {
            log::warn!("Signed raw transaction {:?}",raw_transaction);
            let result = web3_i.eth().send_raw_transaction(raw_transaction.raw).await;
            match result{
                Err(e) => {
                    log::warn!("Node Raw Sending Response {:?}",&e);
                    return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
                },
                Ok(tx_hash) => {
                    log::warn!("to: {:?}, val: {:?}, hash: {:?}",tx_object_clone.to, tx_object_clone.value, result);
                    let tx_data = SendTxData {
                        to:to.to_string(),
                        val:val_u.to_string(),
                        hash:tx_hash.to_string()
                    };
                    let json_res = serde_json::to_string(&tx_data);
                    return (Status::Ok, (ContentType::JSON, json_res.unwrap()));
                }
            }
        }
    }
}


#[openapi(skip)]
#[get("/oldsend/<login>/<recipient>/<volume>")]
pub async fn oldsend(login: &str,
                  recipient: &str,
                  volume: &str,
                  cfg: &State<NodeConfig>,
                  db: Connection<MyDb>) -> (Status, (ContentType, String)) {
    let user = db_functions::pg_query_user(db,login).await.unwrap();
    let user_data : UserData = serde_json::from_value(user.data.unwrap()).unwrap();
    let user_address = &user.address;
    let user_eth_bal = &user_data.balanceEth;
    log::warn!("=======================<SENDING>==========================");
    log::warn!("Username {:?}",login);
    log::warn!("Address {:?}",user_address);
    log::warn!("Balance {:?}",user_eth_bal);
    let nurl: &str = &(cfg.nodeurl.clone() + ":" + &cfg.nodeport.to_string());
    let transport = web3::transports::Http::new(nurl).unwrap();
    let web3 = web3::Web3::new(transport);

    let to = Address::from_str(recipient).unwrap();

    let vol = volume.parse::<f64>().unwrap();
    let vals = ((1000000000000000000.0*vol) as u64).to_string();
    let val_u= U256::from_dec_str(&vals).unwrap();
    log::warn!("Sending value {:?}",val_u);
    let prvk = SecretKey::from_str("0x61079623c42a616efbc9b2f27f42b1c73f08cfd898c60e89589cf15f7bab8b57").unwrap();

    let tx_object = TransactionParameters {
        to: Some(to),
        value: val_u,
        ..Default::default()
    };

    let tx_object_clone = tx_object.clone();

    let r_signed = web3.accounts().sign_transaction(tx_object, &prvk).await;
    match r_signed{
        Err(e) => {
            log::warn!("NodeErr Response {:?}",&e);
            return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
        },
        Ok(signed) => {
            log::warn!("Signed transaction {:?}",signed);
            let result = web3.eth().send_raw_transaction(signed.raw_transaction).await;
            match result{
                Err(e) => {
                    log::warn!("NodeErr Response {:?}",&e);
                    return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
                },
                Ok(tx_hash) => {
                    log::warn!("to: {:?}, val: {:?}, hash: {:?}",tx_object_clone.to, tx_object_clone.value, result);
                    let tx_data = SendTxData {
                        to:to.to_string(),
                        val:val_u.to_string(),
                        hash:tx_hash.to_string()
                    };
                    let json_res = serde_json::to_string(&tx_data);
                    return (Status::Ok, (ContentType::JSON, json_res.unwrap()));
                }
            }
        }
    }

}


#[openapi(tag = "send")]
#[get("/senddummy/eth/login/<login>/<recipient>/<volume>")]
pub async fn senddummy_eth_from_login(login: &str,
                  recipient: &str,
                  volume: &str,
                  cfg: &State<NodeConfig>,
                  db: Connection<MyDb>) -> (Status, (ContentType, String)) {
  let user = db_functions::pg_query_user(db,login).await.unwrap();
  let user_data : UserData = serde_json::from_value(user.data.unwrap()).unwrap();
  let user_address = &user.address;
  let user_eth_bal = &user_data.balanceEth;
  log::warn!("=======================<SENDING>==========================");
  log::warn!("Username {:?}",login);
  log::warn!("Address {:?}",user_address);
  log::warn!("Balance {:?}",user_eth_bal);
  log::warn!("Parsing {:?}",&volume);
  let eth_tx = EthTransactionNoData {
      from : user_address.to_string(),
      to : recipient.to_string(),
      value: to_hex_str(volume),
  };
  let r_send_tx = node_calls::send_eth_tx(cfg,&eth_tx);
  match r_send_tx{
      Err(e) => {
          log::warn!("NodeErr Response {:?}",&e);
          return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
      },
      Ok(send_tx) => {
          let json_res = serde_json::to_string(&send_tx);
          return (Status::Ok, (ContentType::JSON, json_res.unwrap()));
      }
  }
}

#[openapi(tag = "send")]
#[get("/senddummy/eth/address/<address>/<recipient>/<volume>")]
pub async fn senddummy_eth_from_address(address: &str,
                  recipient: &str,
                  volume: &str,
                  cfg: &State<NodeConfig>,
                  db: Connection<MyDb>) -> (Status, (ContentType, String)) {
  log::warn!("=======================<SENDING>==========================");
  log::warn!("Address {:?}",address);
  log::warn!("Recipient {:?}",recipient);
  log::warn!("Parsing {:?}",&volume);
  log::warn!("Hex {:?}",to_hex_str(volume));
  let eth_tx = EthTransactionNoData {
      from : address.clone().to_string(),
      to : recipient.to_string(),
      value: to_hex_str(volume),
  };
  log::warn!("=======================</SENDING>==========================");
  return (Status::Ok, (ContentType::JSON, "Sending eth ".to_owned()+&address.to_string()))
}

/*
#[openapi(skip)]
#[get("/estimatetx/eth/<login>/<recipient>/<volume>")]
pub async fn estimate_eth_transaction(login: &str,
                                  recipient: &str,
                                  volume: &str,
                                  _db: Connection<MyDb>,
                                  cfg: &State<NodeConfig>,
                                  ) -> (Status, (ContentType, String)) {
    log::warn!("Parsing {:?}",&volume);
    let eth_tx = EthTransaction {
        from : login.to_string(),
        to : recipient.to_string(),
        value: to_hex_str(volume),
        gas:"0x15f90".to_string(),
        gasPrice:"0x430e23400".to_string()
    };
    log::warn!("eth_tx: {:?}", eth_tx);
    let r_est_gas = node_calls::estimate_eth_tx_gas(cfg,&eth_tx);
    match r_est_gas{
        Err(e) => {
            log::warn!("NodeErr Response {:?}",&e);
            return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
        },
        Ok(est_gas) => {
            let json_res = serde_json::to_string(&est_gas);
            return (Status::Ok, (ContentType::JSON, json_res.unwrap()));
        }
    }
}
*/
