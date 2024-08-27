use axum::{body::Body, extract::Path, response::{Html as ResponseHtml, IntoResponse, Redirect}, routing::{get,post}, Extension, Form, Router};
use std::{fs, io, sync::Arc};
use tokio::sync::Mutex;
use toker_http::service::ServeDir;
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, SessionManagerLayer};

use components::{error_page, static_page, read_file_to_string};

#[tokio::main]

async fn main() {
    let upload_path = "upload";
    create_directory_if_not_exists(upload_path).unwrap();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::minutes(30)));

    let app = Router::new()
        .route("/", get(home_view))
        .route("/about", get(about_view))
        .route("/contact", get(contact_view))
        .route("services", get(services_view))
        .nest_service("/res", ServeDir::new("res"))
        .nest_service("/uploads", ServeDir::new("uploads"))

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>(),)
            .await
            .unwrap();
}
