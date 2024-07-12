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