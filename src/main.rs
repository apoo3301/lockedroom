#![recursion_limit = "512"]

#![warn(unused_imports)]
use axum::{
    routing::{get, Router},
    http::{StatusCode},
    response::{Html as ResponseHtml, IntoResponse},
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use database::init_db;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::Arc,
};
use thiserror::Error;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

mod database;

#[tokio::main]
#[warn(unused_variables)]
async fn main() {
    let database_conn = Arc::new(Mutex::new(init_db().unwrap()));
    {
        let mut conn = database_conn.lock().await;
    }

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
    println!("server up on port: 3000");
}
