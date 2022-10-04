use crate::types::*;
use crate::node_calls;
use crate::db_functions;
use crate::conf::{NodeConfig, load_config};

use rocket::{get,post,State};
use rocket::http::{Status, ContentType};
use rocket::serde::json::Json;
use rocket_db_pools::{Database, Connection};
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};

#[openapi(tag = "accounts")]
#[get("/createuser/<login>")]
pub async fn user_create(cfg: &State<NodeConfig>, db: Connection<MyDb>, login: &str) -> (Status, (ContentType, String)) {
    let ud : UserData = UserData{
            tokens : [].to_vec(),
            historyEth : [].to_vec(),
            historyTokens : [].to_vec(),
            balanceEth : "0".to_string(),
            balanceTokens : [].to_vec()
            };
    let addr = node_calls::create_new_account(&cfg, login).unwrap();
    db_functions::pg_insert_user(db,login,&addr,&ud).await.unwrap();
    let json_res = serde_json::to_string(&addr);
    return (Status::Ok, (ContentType::JSON, json_res.unwrap()))
}

#[openapi(tag = "accounts")]
#[get("/removeuser/<login>")]
pub async fn user_remove( db: Connection<MyDb>, login: &str) -> Status {
    db_functions::pg_remove_user(db,login).await.unwrap();
    return Status::Ok
}

#[openapi(tag = "accounts")]
#[get("/userdata/<login>")]
pub async fn user_get( db: Connection<MyDb>, login: &str) -> (Status, (ContentType, String)) {
    let user = db_functions::pg_query_user(db,login).await.unwrap();
    let json_res = serde_json::to_string(&user);
    return (Status::Ok, (ContentType::JSON, json_res.unwrap()))
}

#[openapi(tag = "accounts")]
#[get("/allocate_address/<login>")]
pub async fn allocate_address(cfg: &State<NodeConfig>, db: Connection<MyDb>, login: &str) -> (Status, (ContentType, String)) {
    let addr = node_calls::create_new_account(&cfg, login).unwrap();
    let json_res = serde_json::to_string(&addr);
    return (Status::Ok, (ContentType::JSON, json_res.unwrap()))
}

#[openapi(tag = "accounts")]
#[get("/check_address/<login>")]
pub async fn check_address(cfg: &State<NodeConfig>, db: Connection<MyDb>, login: &str) -> (Status, (ContentType, String)) {
    let user = db_functions::pg_query_user(db,login).await.unwrap();
    let json_res = serde_json::to_string(&user.address);
    return (Status::Ok, (ContentType::JSON, json_res.unwrap()))
}

#[openapi(tag = "accounts")]
#[get("/accounts")]
pub async fn accounts_get( _db: Connection<MyDb>, cfg: &State<NodeConfig>) -> (Status, (ContentType, String)) {
    let r_accs = node_calls::get_account_list(cfg);
    match r_accs{
        Err(e) => {
            log::warn!("Node Error {:?}",&e);
            return (Status::InternalServerError,(ContentType::JSON, (&e).to_string()));
        },
        Ok(accs) => {
            let json_res = serde_json::to_string(&accs);
            return (Status::Ok, (ContentType::JSON, json_res.unwrap()));
        }
    }
}

#[openapi(tag = "accounts")]
#[get("/tokens/<login>")]
pub async fn tokens_get( db: Connection<MyDb>, login: &str) -> (Status, (ContentType, String)) {
    let tokens = db_functions::pg_get_user_tokens(db,login).await.unwrap();
    let json_res = serde_json::to_string(&tokens);
    return (Status::Ok, (ContentType::JSON, json_res.unwrap()));
}


#[openapi(tag = "accounts")]
#[post("/tokens/<login>", data = "<tokens>")]
pub async fn tokens_post( db: Connection<MyDb>, login: &str, tokens: Json<Vec<Erc20Token>>) -> Status {
    db_functions::pg_update_user_tokens(db,login,tokens.into_inner()).await.unwrap();
    return Status::Ok
}

#[openapi(skip)]
#[get("/createtest")]
pub async fn createtest(db: Connection<MyDb>) -> String {
    let ud : UserData = UserData{
            tokens : [].to_vec(),
            historyEth : [].to_vec(),
            historyTokens : [].to_vec(),
            balanceEth : "0".to_string(),
            balanceTokens : [].to_vec()
            };
    let log = "test".to_string();
    let addr = "0x9297DB28EeB0bdE3710568cb003982F257eA1cE0".to_string();
    db_functions::pg_insert_user(db,&log,&addr,&ud).await.unwrap();
    return format!("Added test");
}

#[openapi(skip)]
#[get("/updatetokens/<login>")]
pub async fn updatetokens(db: Connection<MyDb>, login: &str) -> String {
    let tokens = [Erc20Token{ticker: "USDT".to_string()
                            ,name: "USDT".to_string()
                            ,contract: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string()
                        },
                  Erc20Token{ticker: "GTECH".to_string()
                            ,name: "GTECH".to_string()
                            ,contract: "0x866A4Da32007BA71aA6CcE9FD85454fCF48B140c".to_string()
                        },
                  Erc20Token{ticker: "CRV".to_string()
                            ,name: "CRV".to_string()
                            ,contract: "0xd533a949740bb3306d119cc777fa900ba034cd52".to_string()
                        },
                ].to_vec();
    db_functions::pg_update_user_tokens(db,login,tokens).await.unwrap();
    return format!("Added test");
}
