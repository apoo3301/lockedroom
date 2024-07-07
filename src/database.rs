use axum_typed_multipart::TypedMultipart;
use chrono::prelude::*;
use image::io::Reader;
use passwd_auth::generate_hash;
use ravif::{Encoder, Img, RGBA8};
use rusqlite::{params, Connection};
use std::fs;
use std::io::Cursor;

#[derive(Debug)]

pub fn init_db() -> Result<Connection, rusqlite::Error> {
	let connection = Connection::open("lockedroom.db")?;

	//users table
	connection.execute(
		"CREATE TABLE IF NOT EXISTS users (
			id INTEGER PRIMARY KEY,
			username TEXT NOT NULL UNIQUE,
			password TEXT NOT NULL,
			created_at TEXT NOT NULL
		)",
		[],
	)?;

	//posts table
	connection.execute(
		"CREATE TABLE IF NOT EXISTS posts (
			post_id INT NOT NULL,
			target_id INT NOT NULL,
			FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
			FOREIGN KEY (target_id) REFERENCES posts(id) ON DELETE CASCADE
		)",
		[],
	)?;

	//mod table
	connection.execute(
		"CREATE TABLE IF NOT EXISTS users (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			username TEXT NOT NULL,
			password TEXT NOT NULL,
			level INT NOT NULL
		)",
		[],
	)?;

	//ban table

	connection.execute(
		"CREATE TABLE IF NOT EXISTS bans (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			username TEXT NOT NULL,
			reason TEXT NOT NULL,
			created_at TEXT NOT NULL
			ip TEXT NOT NULL
		)",
		[],
	)?;

	Ok(connection)
	println!("Database initialized");
}