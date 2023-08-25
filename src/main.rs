use axum::{routing::get, Router, routing::post, extract};
use serde::Deserialize;

async fn hello_world() -> &'static str {
    "Testing custom webhooks."
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Ping {
    zen: String,
    hook_id: i64,
    hook: serde_json::Value
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Events {
    Ping(Ping)
}

async fn handle_event(extract::Json(payload): extract::Json<Events>) -> String {
    match payload {
        Events::Ping(_) => return "Pong".to_string()
    }
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/", post(handle_event));

    Ok(router.into())
}
