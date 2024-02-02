use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

pub fn apply_migrations(connection: &mut Connection) {
    let migrations = Migrations::new(vec![
        M::up(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT, 
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                endpoint TEXT,
                region TEXT,
                access_key TEXT,
                secret_key TEXT
            );",
        ),
        M::up(
            "CREATE TABLE tokens (
                token TEXT NOT NULL, 
                user_id INTEGER NOT NULL,
                FOREIGN KEY (user_id)
                    REFERENCES users (id) 
            );",
        ),
    ]);

    migrations.to_latest(connection).unwrap();
}
