#![recursion_limit="512"]

use axum::{body::Body, http::{Response, StatusCode}, response::{HTML as ResponseHTML, IntoResponse, Redirect}, routing::{get, post}, Extension, Form, Router};
use database::{ init_db, create_user, delete_user, get_users, get_user_by_id, get_user_by_name};
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, SessionManagerLayer};
use std::{net::SocketAddr, sync::Arc};
use tower_http::services::ServeDir;
use rusqlite::Connection;
use tokio::sync::Mutex;
use thiserror::Error;
use serde::Serialize;
use bytes::Bytes;

// Define other
// use components::{ admin_page, error_page, login_page, main_page, post_page, read_file_to_string, static_page };
// use axum_login::{login_required, permission_required, AuthManagerLayerBuilder};
// use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
// use crate::database::create_post;
// use auth::{Backend, StaffPerms}
// use crate::auth::Credentials;

mod database;

// type AuthSession = axum_login::AuthSession<Backend>;

#[tokio::main]
async fn main() {
    let database_connection = Arc::nex(Mutex::new(init_db().unwrap().expect("Failed to initialize database.")));
    {
        let connection = database_connection.lock().await;
    }

    let session_store = MemoryStore::default();
    let session_manager = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::Timeout(Duration::minutes(30)));
    let backend = Backend::new(database_connection.clone());
    let auth_manager = AuthManagerLayerBuilder::new(backend)
        .with_session_manager(session_manager)
        .build();
    let app = Router::new();
        .route("/", get(main_page))
}

async fn main_page(Extension(database_connection): Extension<Arc<Mutex<Connection>>>) -> impl IntoResponse {
    let database_connection = database_connection.lock().await;
    ResponseHTML(include_str!("../static/main.html"))
}

async fn post_main_handler(Extension(database_connection): Extension<Arc<Mutex<Connection>>>) -> Response<Body> {
    let ip = addr.ip().to_string();
    let mut database_connection = database_connection.lock().await;
}