use axum::{
    body::Body,
    extract::Path,
    response::{Html as ResponseHtml, IntoResponse, Redirect},
    routing::{get, post},
    Extension, Form, Router,
};
use std::{fs, io, sync::Arc};
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, SessionManagerLayer};

use components::{error_page, static_page, read_file_to_string};

#[tokio::main]
async fn main() {
    let upload_path = "uploads";
    create_directory_if_not_exists(upload_path).unwrap();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    let app = Router::new()
        // Main views
        .route("/", get(main_view))
        .route("/about", get(about_view))
        .route("/contact", get(links_view))
        .route("/service", get(conduct_view))
        // Static files
        // .nest_service("/res", ServeDir::new("res"))
        // .nest_service("/uploads", ServeDir::new("uploads"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}

fn create_directory_if_not_exists(path: &str) -> io::Result<()> {
    if !fs::metadata(path).is_ok() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

async fn main_view() -> impl IntoResponse {
    ResponseHtml("<h1>Welcome to the main page</h1>")
}

async fn about_view() -> impl IntoResponse {
    let content = read_file_to_string("static/about.html");
    ResponseHtml(static_page(
        content.unwrap_or("Cette page n'existe pas".to_owned()),
    ))
}

async fn contact_view() -> impl IntoResponse {
    let content = read_file_to_string("static/contact.html");
    ResponseHtml(static_page(
        content.unwrap_or("Cette page n'existe pas".to_owned()),
    ))
}

async fn service_view() -> impl IntoResponse {
    let content = read_file_to_string("static/service.html");
    ResponseHtml(static_page(
        content.unwrap_or("Cette page n'existe pas".to_owned()),
    ))
}
