use axum_typed_multipart::TypedMultipart;
use chrono::prelude::*;
use image::io::Reader;
use password_auth::generate_hash;
use ravif::{Encoder, Img, RGBA8};
use rusqlite::{params, Connection};
use std::fs;
use std::io::Cursor;

#[derive(Debug)]
pub struct Post {
    pub id: i32,
    pub time: i64,
    pub nick: String,
    pub body: String,
    pub author: String,
    pub upload: Option<i64>,
    pub parent: Option<i32>,
}

pub struct Mention {
    pub post_id: i32,
    pub target_id: i32,
}

#[derive(Debug, PartialEq)]
pub struct UserBan {
    pub ip: String,
    pub reason: Option<String>,
}

pub fn init_db() -> Result<Connection, rusqlite::Error> {
    let connection = Connection::open("lockedroom.db")?;
    println!("connected to database.");

    connection.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            password TEXT NOT NULL,
            level INTEGER NOT NULL
        )",
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            time INTEGER NOT NULL,
            nick TEXT NOT NULL,
            body TEXT NOT NULL,
            author TEXT NOT NULL,
            upload INTEGER,
            parent INTEGER,
            FOREIGN KEY (parent) REFERENCES posts(id) ON DELETE CASCADE
        )",
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS mods (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            password TEXT NOT NULL,
            level INTEGER NOT NULL
        )",
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS bans (
            ip TEXT NOT NULL,
            reason TEXT
        )",
        [],
    )?;

    Ok(connection)
}


pub fn create_user(
    conn: &Connection,
    username: &str,
    password: &str,
    level: &i64,
) -> Result<usize, rusqlite::Error> {
    let password = generate_hash(password);
    conn.execute(
        "INSERT INTO users (username, password, level) VALUES (?1, ?2, ?3)",
        params![username, password, level],
    )
}


pub fn delete_user(conn: &Connection, id: i32) -> Result<usize, rusqlite::Error> {
	conn.execute("DELETE FROM users WHERE username = ?1", params![id])
}