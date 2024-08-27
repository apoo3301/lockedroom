use crate::{auth::User, parser::grab_mentions, CreatePostError, CreatePostRequest};
use axum_type_multipart::TypedMultipart;
use image::io::Reader;
use passwd_auth::generate_hash;
use ravif::{Encore, Img, RGBA8};
use rusqlite::{params, Connection};
use std::fs;
use std::io::Cursor;

mod config;

#[derive(Debug)]
pub struct Post {
    pub id: i32,
    pub time: i64,
    pub author: String,
    pub content: String,
    pub username: String,
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

pub fn init_database() -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open("lockedroom.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY,
            time INTEGER NOT NULL,
            author TEXT NOT NULL,
            content TEXT NOT NULL,
            username TEXT NOT NULL,
            upload INTEGER,
            parent INTEGER
            FOREIGN KEY (parent) REFERENCES posts(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS mentions (
            post_id INTEGER NOT NULL,
            target_id INTEGER NOT NULL
            FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE
            FOREIGN KEY (target_id) REFERENCES posts(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            password TEXT NOT NULL,
            level INTEGER NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bans (
            ip TEXT PRIMARY KEY,
            reason TEXT
        )",
        [],
    )?;
    Ok(conn)
}

pub fn create_mention(conn: &Connection, post_id: i64, target_id: i32) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "INSERT INTO mentions (post_id, target_id) VALUES (?1, ?2)",
        params![post_id, target_id],
    )
}

pub fn create_ban(conn: &Connection, ip: &str, reason: Option<String>) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "INSERT INTO bans (ip, reason) VALUES (?1, ?2)",
        params![ip, reason],
    )
}

pub fn delete_ban(conn: &Connection, ip: &str) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "DELETE FROM bans WHERE ip = ?1",
        params![ip],
    )
}

pub fn get_ban(conn: &Connection, ip: &str) -> Result<UserBan, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM bans WHERE ip = ?1")?;
    let mut rows = stmt.query(params![ip])?;
    let row = rows.next().unwrap()?;
    Ok(UserBan {
        ip: row.get(0)?,
        reason: row.get(1)?,
    })
}

pub fn create_user(conn: &Connection, username: &str, password: &str, level: i64) -> Result<usize, rusqlite::Error> {
    let password = generate_hash(password);
    conn.execute(
        "INSERT INTO users (username, password, level) VALUES (?1, ?2, ?3)",
        params![username, password, level],
    )
}

pub fn delete_user(conn: &Connection, username: &str) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "DELETE FROM users WHERE username = ?1",
        params![username],
    )
}

pub fn get_mentions_by_target(conn: &Connection, target_id: i32) -> Result<Vec<Mention>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM mentions WHERE target_id = ?1")?;
    let rows = stmt.query_map(params![target_id], |row| {
        Ok(Mention {
            post_id: row.get(0)?,
            target_id: row.get(1)?,
        })
    })?;
    let mut mentions = Vec::new();
    for mention in rows {
        mentions.push(mention?);
    }
    Ok(mentions)
}

//config::STALE_THREADS_TIME_THRESHOLD
//config::STALE_THREADS_OFFSET

fn cull_threads(conn: &mut Connection) -> Result<usize, rusqlite::Error> {
    let tx = conn.transaction()?;

    tx.execute(format!("
    CREATE TEMP VIEW sel_threads AS
    SELECT thread.id, thread.upload
    FROM posts thread
    LEFT JOIN posts post ON post.parent = thread.id
    WHERE threat.parent IS NULL
    ADD ((post.id IS NULL AND thread.time < strftime('%s', 'now') - {0}) OR post.time < strftime('%s', 'now') - {0})
    ORDER BY thread.time DESC
    LIMIT -1 OFFSET {1}", config::STALE_THREADS_TIME_THRESHOLD, config::STALE_THREADS_OFFSET).as_str(), [])?;

    let mut stmt = tx.prepare("
        SELECT t.upload
        FROM sel_threads t
        WHERE t.upload IS NOT NULL
        UNION
        SELECT p.upload
        FROM sel_threads t
        INNER JOIN POSTS p ON p.parent = t.id
        WHERE p.upload IS NOT NULL")?;

    let images_to_delete: Vec<i64>;
    let rows = stmt.query_map([], |row| { row.get::<_, i64>(0) })?;
    images_to_delete = rows.filter_map(|row| row.ok()).collect();

    tx.execute("
        DELETE FROM posts
        WHERE id IN (
            SELECT p1.id
            FROM posts p1
            LEFT JOIN posts p2 ON p2.parent = p1.id
            WHERE p1.parent IS NULL
            AND ((p2.id IS NULL AND p1.time < strfime('%s', 'now') - ?1) OR p2.time < strftime('%s', 'now') - ?1)
            ORDER BY p1.time DESC
            LIMIT -1 OFFSET ?2
        );", params!(config::STALE_THREADS_TIME_THRESHOLD, config::STALE_THREADS_OFFSET)
    )?;

    stmt.finalize()?;
    let commit = tx.commit()?;

    if commit.is_ok() {
        for img in images_to_delete {
            let file_path = format!("uploads/{}.avif", img);
            let _ = fs::remove_file(file_path);
        }
    };
    commit
}

