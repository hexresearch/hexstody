use hexstody_db::Pool;
use hexstody_db::state::State;
use rweb::openapi::Spec;
use rweb::*;
use serde::Serialize;
use std::convert::From;
use std::convert::Infallible;
use std::error::Error;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

// impl rweb::reject::Reject for queries::Error {}

#[get("/ping")]
#[openapi(
    tags("helpers"),
    summary = "Returns 200 if all systems are OK",
    description = "Successfull call to ping indicates that the API is fully operational and you can use other methods of the API."
)]
async fn ping_endpoint(
    #[data] pool: Pool,
) -> Result<Json<()>, Rejection> {
    Ok(Json::from(()))
}

pub async fn public_api_specs(pool: Pool) -> Result<Spec, Box<dyn Error>> {
    let (spec, _) = openapi::spec().build(|| {
        ping_endpoint(pool)
            .recover(handle_rejection)
    });
    Ok(spec)
}

pub async fn serve_public_api(
    host: &str,
    port: u16,
    pool: Pool,
    state: Arc<Mutex<State>>,
    state_notify: Arc<Notify>,
) -> Result<(), Box<dyn Error>> {
    let filter = ping_endpoint(pool)
        .recover(handle_rejection)
        .with(log("hexstody::api"));
    serve(filter).run((IpAddr::from_str(host)?, port)).await;
    Ok(())
}

/// An API error serializable to JSON.
#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

// This function receives a `Rejection` and tries to return a custom
// value, otherwise simply passes the rejection along.
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    // } else if let Some(err) = err.find::<queries::Error>() {
    //     error!("Rejection by query fail: {}", err);
    //     code = StatusCode::BAD_REQUEST;
    //     message = "SERVER_DATABASE_ERROR";
    } else if let Some(e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        // This error happens if the body could not be deserialized correctly
        // We can use the cause to analyze the error and customize the error message
        message = match e.source() {
            Some(cause) => {
                if cause.to_string().contains("denom") {
                    "FIELD_ERROR: denom"
                } else {
                    "BAD_REQUEST"
                }
            }
            None => "BAD_REQUEST",
        };
        code = StatusCode::BAD_REQUEST;
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hexstody_client::client::HexstodyClient;
    use futures::FutureExt;
    use futures_util::future::TryFutureExt;
    use std::panic::AssertUnwindSafe;

    const SERVICE_TEST_PORT: u16 = 8198;
    const SERVICE_TEST_HOST: &str = "127.0.0.1";

    async fn run_api_test<F, Fut>(pool: Pool, test_body: F)
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = ()>,
    {
        let _ = env_logger::builder().is_test(true).try_init();

        let state_mx = Arc::new(Mutex::new(State::default()));
        let state_notify = Arc::new(Notify::new());

        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn({
            let state = state_mx.clone();
            let state_notify = state_notify.clone();
            async move {
                let serve_task = serve_public_api(
                    SERVICE_TEST_HOST,
                    SERVICE_TEST_PORT,
                    pool,
                    state,
                    state_notify,
                );
                futures::pin_mut!(serve_task);
                futures::future::select(serve_task, receiver.map_err(drop)).await;
            }
        });

        let res = AssertUnwindSafe(test_body()).catch_unwind().await;

        sender.send(()).unwrap();

        assert!(res.is_ok());
    }

    #[sqlx_database_tester::test(pool(
        variable = "pool",
        migrations = "../hexstody-db/migrations"
    ))]
    async fn test_public_api_ping() {
        run_api_test(
            pool,
            || async {
                let client = HexstodyClient::new(&format!(
                    "http://{}:{}",
                    SERVICE_TEST_HOST, SERVICE_TEST_PORT
                ));
                client
                    .ping()
                    .await
                    .unwrap();
            },
        )
        .await;
    }
}
