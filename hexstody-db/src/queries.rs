use super::state::*;
use super::update::*;
use super::Pool;
use chrono::prelude::*;
use futures::StreamExt;
use hexstody_api::error::HexstodyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Failed to decode body by tag: {0}")]
    UpdateBody(#[from] UpdateBodyError),
    #[error("Failed to decode/encode JSON: {0}")]
    Encoding(#[from] serde_json::Error),
    #[error("Failed to reconstruct state: {0}")]
    StateInvalid(#[from] crate::error::StateUpdateErr),
}

impl HexstodyError for Error {
    fn subtype() -> &'static str {
        "hexstody-db:queries"
    }

    fn code(&self) -> u16 {
        match self {
            Error::Database(_) => 0,
            Error::UpdateBody(_) => 1,
            Error::Encoding(_) => 2,
            Error::StateInvalid(_) => 3,
        }
    }

    fn status(&self) -> u16 {
        500
    }
}

/// Alias for a `Result` with the error type `self::Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// Query all history of updates until we hit a snapshot or the begining of time
pub async fn query_updates(pool: &Pool) -> Result<Vec<StateUpdate>> {
    let mut conn = pool.acquire().await?;
    let res = sqlx::query!("select * from updates order by created desc")
        .fetch(&mut conn)
        .fuse();
    futures::pin_mut!(res);

    let mut parsed: Vec<StateUpdate> = vec![];
    loop {
        let item = futures::select! {
            mmrow = res.next() => {
                if let Some(mrow) = mmrow {
                    let r = mrow?;
                    let body = UpdateTag::from_tag(&r.tag, r.version as u16, r.body.clone())?;
                    StateUpdate {
                        created: r.created,
                        body,
                        callback_channel: None
                    }
                } else {
                    break;
                }
            },
            complete => break,
        };
        let is_end = item.body.tag() == UpdateTag::Snapshot;
        parsed.push(item);
        if is_end {
            break;
        }
    }
    Ok(parsed)
}

/// Insert new update in the chain of updates in database
pub async fn insert_update(
    pool: &Pool,
    update: UpdateBody,
    timestamp: Option<NaiveDateTime>,
) -> Result<()> {
    let now = timestamp.unwrap_or_else(|| Utc::now().naive_utc());
    let tag = format!("{}", update.tag());
    let body = update.json()?;
    sqlx::query!(
        "insert into updates (created, version, tag, body) values ($1, $2, $3, $4)",
        now,
        CURRENT_BODY_VERSION as i16,
        tag,
        body
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Reconstruct state from chain of updates and snapshots in the database
pub async fn query_state(network: Network, pool: &Pool) -> Result<State> {
    let updates = query_updates(pool).await?;
    Ok(State::collect(network, updates.into_iter().rev())?)
}

#[cfg(test)]
mod tests {
    #[sqlx_database_tester::test(
        pool(variable = "migrated_pool", migrations = "./migrations"),
        pool(
            variable = "empty_db_pool",
            transaction_variable = "empty_db_transaction",
            skip_migrations
        )
    )]
    async fn test_server_start() {
        let migrated_pool_tables = sqlx::query!("SELECT * FROM pg_catalog.pg_tables")
            .fetch_all(&migrated_pool)
            .await
            .unwrap();
        let empty_pool_tables = sqlx::query!("SELECT * FROM pg_catalog.pg_tables")
            .fetch_all(&empty_db_pool)
            .await
            .unwrap();
        println!("Migrated pool tables: \n {:#?}", migrated_pool_tables);
        println!("Empty pool tables: \n {:#?}", empty_pool_tables);
    }
}
