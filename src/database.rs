use rusqlite::Connection;

pub fn init_db() -> Result<Connection, rusqlite::Error> {
    let connection = Connection::open("lockedroom.db")?;
    println!("connected to database.");

    connection.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            post_id INTEGER PRIMARY KEY,
            target_id INTEGER NOT NULL,
            FOREIGN KEY (post_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (target_id) REFERENCES users(id) ON DELETE CASCADE
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
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            reason TEXT NOT NULL,
            created_at TEXT NOT NULL,
            ip TEXT NOT NULL
        )",
        [],
    )?;

    Ok(connection)
}
