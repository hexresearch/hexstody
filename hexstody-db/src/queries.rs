use super::Pool;
use chrono::prelude::*;
use futures::StreamExt;
use super::state::*;
use super::update::*;
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
    StateInvalid(#[from] StateUpdateErr),
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
                        body
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
pub async fn insert_update(pool: &Pool, update: UpdateBody) -> Result<()> {
    let now = Utc::now().naive_utc();
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
pub async fn query_state(pool: &Pool) -> Result<State> {
    let updates = query_updates(pool).await?;
    Ok(State::collect(updates.into_iter().rev())?)
}