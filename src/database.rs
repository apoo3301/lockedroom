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

pub fn create_post(conn: &mut Connection, ip: &str, data: TypedMultipart<CreatePostRequest>, parent: Option<i32>) -> Result<i64, CreatePostError> {
    let now = Utc::now().timestamp();

    let username = Some(data.username.trim())
        .filter(|username| !username.is_empty())
        .unwrap_or_else(|| "vagabond");

    if data.message.trim().is_empty() == true {
        Err(CreatePostError::InvalidForm)?;
    }

    if data.message.len() > 7000 {
        Err(CreatePostError::InvalidForm)?;
    }

    let mut query = "INSERT INTO posts (time, author, content, username, parent, upload) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";

    if let Some(parent) = parent {
        match get_post_by_id(conn, parent) {
            Ok(_) => {
                query = "INSERT INTO posts (time, author, content, username, parent, upload) 
                SELECT ?1, ?2, ?3, ?4, ?5, ?6
                WHERE (SELECT COUNT(*) FROM posts WHERE parent = ?6) < 128;"
            }
            Err(_) => {
                Err(CreatePostError::MissingThread)?;
            }
        } else {
            let _cull = cull_threads(conn);
        }

        let upload_contents = data
            .upload
            .as_ref()
            .filter(|upload| upload.content.len() > 0)
            .map(|upload| -> Result<_, CreatePostError> {
                let allowed_content_types = [Some("image/jpeg"), Some("image/png")];
                let content_type = &upload.metadata.content_type.as_deref();
                if !allowed_content_types.contains(content_type) {
                    Err(CreatePostError::InvalidUpload)?;
                }
                Ok(&upload.contents)
            })
            .transpose()?;

        let upload_id = upload_contents
            .map(|contents| match save_upload(conn, contents) {
                Ok(id) => Ok(id),
                Err(SaveUploadError::DecodeError) => Err(CreatePostError::InvalidUpload),
                Err(SaveUploadError::EncodeError) => Err(CreatePostError::Interal),
                Err(SaveUploadError::WriteError) => Err(CreatePostError::Internal),
            })
            .transpose()?;
        
        let tx = conn.transaction().unwrap();

        let exec = match tx.execute(
            query,
            params![now, ip, data.message, username, parent, upload_id],
        ) {
            Ok(ins) => {
                if ins == 0 {
                    Err(CreatePostError::ReplyLimit);
                };
                Ok(tx.last_insert_rowid())
            }
            Err(_) => Err(CreatePostError::Internal),
        };

        let _ = match tx.commit() {
            Ok(_) => (),
            Err(_) => Err(CreatePostError::Internal)?,
        };
        
        match exec {
            Ok(post_id) => {
                let mentions: Vec<i32> = grab_mentions(data.message.as_str(), conn);
                for target_id in mentions {
                    let _ = create_mention(conn, post_id, target_id);
                }

                Ok(post_id)
            }
            Err(_) => Err(CreatePostError::Internal)?,
        }
    }
}

#[derive(Debug)]
enum SaveUploadError {
    DecodeError,
    EncodeError,
    WriteError,
}
//save_upload, populate_sample_data,

fn get_posts(conn: &Connection, query: String) -> Result<Vec<Post>, rusqlite::Error> {
    let mut statement = conn.prepare(query.as_str())?;
    let p_iter = statement.query_map([], |row| {
        Ok(Post {
            id: row.get(0)?,
            time: row.get(1)?,
            username: row.get(2)?,
            content: row.get(3)?,
            author: row.get(4)?,
            upload: row.get(5)?,
            parent: row.get(6)?,
        })
    })?;

    let mut posts = Vec::new();
    for post in p_iter {
        posts.push(post?);
    }
    Ok(posts)
}