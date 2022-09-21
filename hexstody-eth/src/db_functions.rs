use crate::types::*;

use sqlx::postgres::{PgPoolOptions};
use rocket_db_pools::{sqlx, Connection};



pub async fn create_db_pool(conn_string: &str) -> Result<Pool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(conn_string)
        .await?;

    Ok(pool)
}

pub async fn pg_query_users(pool: &Pool) -> Result<Vec<UserEth>, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    let users = sqlx::query_as!(
            UserEth,
            "SELECT * FROM users_eth",
        )
        .fetch_all(&mut conn)
        .await?;
    Ok(users)
}

pub async fn pg_query_user(db: Connection<MyDb>, login: &str) -> Result<UserEth, sqlx::Error> {
    let user = sqlx::query_as!(
            UserEth,
            "SELECT * FROM users_eth WHERE login=$1",
            login
        )
        .fetch_one(&mut db.into_inner())
        .await?;
    Ok(user)
}

pub async fn pg_insert_user(db: Connection<MyDb>, login: &str, address: &str, data: &UserData)
                            -> Result<(), sqlx::Error> {
    let data_json = serde_json::to_value(data).unwrap();
    sqlx::query!(
            "INSERT INTO users_eth (login, address, data) values ($1, $2, $3)",
            login,
            address,
            data_json
        )
        .execute(&mut db.into_inner())
        .await?;
    Ok(())
}

pub async fn pg_remove_user(db: Connection<MyDb>, login: &str)
                            -> Result<(), sqlx::Error> {
    sqlx::query!(
            "DELETE FROM users_eth WHERE login=$1",
            login
        )
        .execute(&mut db.into_inner())
        .await?;
    Ok(())
}

pub async fn pg_update_user_tokens(db: Connection<MyDb>, login: &str, tokens: Vec<Erc20Token>)
                            -> Result<(), sqlx::Error> {
    let data_json = serde_json::to_value(tokens).unwrap();
    sqlx::query!(
            "UPDATE users_eth SET data = jsonb_set(data, '{tokens}', $2) WHERE login = $1",
            login,
            data_json
        )
        .execute(&mut db.into_inner())
        .await?;
    Ok(())
}

pub async fn pg_get_user_tokens(db: Connection<MyDb>, login: &str)
                            -> Result<Vec<Erc20Token>, sqlx::Error> {
    let dbresp = sqlx::query_as!(
            Erc20TokenWrapper,
            "SELECT ((data::json->'tokens')::jsonb) as tokens FROM users_eth WHERE login = $1",
            login
        )
        .fetch_one(&mut db.into_inner())
        .await?;

    let tks :Vec<Erc20Token> = serde_json::from_value(dbresp.tokens.unwrap()).unwrap();
    Ok(tks)
}

#[allow(dead_code)]
pub async fn pg_update_user(db: Connection<MyDb>, login: &str, data: UserData)
                            -> Result<(), sqlx::Error> {
    let data_json = serde_json::to_value(data).unwrap();
    sqlx::query!(
            "UPDATE users_eth SET data = $2 WHERE login = $1",
            login,
            data_json
        )
        .execute(&mut db.into_inner())
        .await?;
    Ok(())
}

pub async fn pg_update_user_old(pool: &Pool, login: &str, data: UserData)
                            -> Result<(), sqlx::Error> {
    let mut conn = pool.acquire().await?;
    let data_json = serde_json::to_value(data).unwrap();
    sqlx::query!(
            "UPDATE users_eth SET data = $2 WHERE login = $1",
            login,
            data_json
        )
        .execute(&mut conn)
        .await?;
    Ok(())
}


pub async fn pg_query_total_eth(db: Connection<MyDb>) ->  Result<Vec<String>, sqlx::Error> {
    let wrapped_eth_balanaces = sqlx::query_as!(
            EthBalanceWrapper,
            "SELECT ((data::json->'balanceEth')::jsonb) as balance FROM users_eth",
            )
        .fetch_all(&mut db.into_inner())
        .await?;
    let mut balances : Vec<String> = [].to_vec();
    for wbal in wrapped_eth_balanaces{
        let bal : String = serde_json::from_value(wbal.balance.unwrap()).unwrap();
        balances.push(bal)
    }
    Ok(balances)
}

pub async fn pg_query_total_tokens(db: Connection<MyDb>) ->  Result<Vec<Vec<Erc20TokenBalance>>, sqlx::Error> {
    let wrapped_eth_balanaces = sqlx::query_as!(
            Erc20TokenBalanceWrapper,
            "SELECT ((data::json->'balanceTokens')::jsonb) as erc20_balance FROM users_eth",
            )
        .fetch_all(&mut db.into_inner())
        .await?;
    let mut balances : Vec<Vec<Erc20TokenBalance>> = [].to_vec();
    for wbal in wrapped_eth_balanaces{
        let bal : Vec<Erc20TokenBalance> = serde_json::from_value(wbal.erc20_balance.unwrap()).unwrap();
        balances.push(bal)
    }
    Ok(balances)
}
