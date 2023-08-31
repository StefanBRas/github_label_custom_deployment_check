use axum::{
    body::{Body, Bytes},
    extract,
    extract::FromRequestParts,
    extract::{BodyStream, State},
    http::{header, request::Parts, Request, StatusCode},
    middleware::from_extractor,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    routing::post,
    Router,
};
use hyper::body::to_bytes;

use anyhow::anyhow;
use github_webhook_message_validator::validate;

use serde::Deserialize;
use shuttle_secrets::SecretStore;

async fn hello_world() -> &'static str {
    "Testing custom webhooks."
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Ping {
    zen: String,
    hook_id: i64,
    hook: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(untagged)]
// use: X-GitHub-Event: deployment_protection_rule
enum Events {
    Ping(Ping),
    Json(serde_json::Value),
}

async fn handle_event(extract::Json(payload): extract::Json<Events>) -> String {
    match payload {
        Events::Ping(_) => return "Pong".to_string(),
        Events::Json(_) => return "Nothing to do".to_string(),
    }
}

#[derive(Clone)]
struct AppState {
    webhook_pass: String,
}

async fn my_middleware(
    State(state): State<AppState>,
    // you can add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    request: Request<Body>,
    next: Next<Body>,
) -> Result<Response, StatusCode> {
    let signature = request
        .headers()
        .get("x-hub-signature-256")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .as_bytes();
    let b = to_bytes(request.body()).await.map_err(|_|StatusCode::UNAUTHORIZED)?;
    if validate(
        state.webhook_pass.as_bytes(),
        signature,
        &b) {
        let response = next.run(request).await;
        Ok(response)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[shuttle_runtime::main]
async fn axum(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let webhook_pass = if let Some(secret) = secret_store.get("5VTqexM8A4dy7Nczzre459SMUGWPdrWP") {
        secret
    } else {
        return Err(anyhow!("secret was not found").into());
    };

    let state = AppState { webhook_pass };

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/", post(handle_event))
        .route_layer(middleware::from_fn_with_state(state.clone(), my_middleware))
        .with_state(state);

    Ok(router.into())
}
