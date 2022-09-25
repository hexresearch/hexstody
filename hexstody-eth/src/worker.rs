use std::time::Duration;
use crate::db_functions;
use crate::node_calls;
use crate::types::*;


pub async fn node_worker(
    polling_sleep: Duration,
    pool: &Pool
) -> () {
    loop {
        {
            match update_stage(pool).await {
                Ok(str) => {
                    log::info!("Debug, got Ok(): {str}");
                }
                Err(e) => {
                    log::warn!("Error: Failed to query node: {e}");
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            }
        }
        tokio::time::sleep(polling_sleep).await;
    }
}

pub async fn update_stage(pool: &Pool) -> Result<String, sqlx::Error> {
    let users = db_functions::pg_query_users(&pool).await?;
    for u in users{
        tokio::time::sleep(Duration::from_secs(1)).await;
        let slog = u.login;
        let saddr = u.address;
        let sdata : UserData = serde_json::from_value(u.data.unwrap()).unwrap();
        let stokens = &sdata.tokens;
        let mut hist_tokens = [].to_vec();
        let mut bal_tokens  = [].to_vec();
        let sbaleth = node_calls::get_balance_eth(&saddr).await.unwrap();
        let sethhist = node_calls::get_history_eth(&saddr).await.unwrap();
        for t in stokens{
            tokio::time::sleep(Duration::from_secs(1)).await;
            let token_history = node_calls::get_history_token(&saddr,&t.contract,&t.ticker).await.unwrap();
            let token_balance = node_calls::get_balance_token(&saddr,&t.contract,&t.ticker).await.unwrap();
            hist_tokens.push(token_history);
            bal_tokens.push(token_balance);
        }
        let user_data_updated : UserData = UserData{
                tokens : sdata.tokens,
                historyEth : sethhist,
                historyTokens : hist_tokens,
                balanceEth : sbaleth,
                balanceTokens : bal_tokens
                };
        db_functions::pg_update_user_old(&pool,&slog,user_data_updated).await.unwrap();
    }
    return Ok("Getted users".into());
}
