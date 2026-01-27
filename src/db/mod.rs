use duckdb::{params, Connection};
use anyhow::Context;
use std::sync::{Arc, Mutex};

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Debug, Clone)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct PlaylistEntry {
    pub id: i64,
    pub playlist_id: i64,
    pub url: String,
    pub title: String,
    pub position: i32,
}

impl Database {
    pub fn new() -> anyhow::Result<Self> {
        let path = "playlists.db";
        let conn = Connection::open(path).context("Failed to open DuckDB connection")?;

        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.init().context("Failed to initialize database schema")?;

        Ok(db)
    }

    pub fn init(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            r"
            CREATE SEQUENCE IF NOT EXISTS seq_playlists_id;
            CREATE SEQUENCE IF NOT EXISTS seq_playlist_entries_id;

            CREATE TABLE IF NOT EXISTS playlists (
                id BIGINT PRIMARY KEY DEFAULT nextval('seq_playlists_id'),
                name TEXT NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS playlist_entries (
                id BIGINT PRIMARY KEY DEFAULT nextval('seq_playlist_entries_id'),
                playlist_id BIGINT NOT NULL,
                url TEXT NOT NULL,
                title TEXT NOT NULL,
                position INTEGER NOT NULL,
                FOREIGN KEY(playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
            );
            "
        ).context("Failed to execute DB init batch")?;

        Ok(())
    }

    pub fn create_playlist(&self, name: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO playlists (name) VALUES (?)",
            [name],
        ).context("Failed to insert playlist")?;
        Ok(())
    }

    pub fn get_playlists(&self) -> anyhow::Result<Vec<Playlist>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM playlists ORDER BY name ASC")?;

        let playlist_iter = stmt.query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;

        let mut playlists = Vec::new();
        for playlist in playlist_iter {
            playlists.push(playlist?);
        }

        Ok(playlists)
    }

    pub fn add_song(&self, playlist_id: i64, url: &str, title: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        // Get max position
        let max_pos: Option<i32> = conn.query_row(
            "SELECT MAX(position) FROM playlist_entries WHERE playlist_id = ?",
            [playlist_id],
            |row| row.get(0),
        ).unwrap_or(None);

        let new_pos = max_pos.unwrap_or(0) + 1;

        conn.execute(
            "INSERT INTO playlist_entries (playlist_id, url, title, position) VALUES (?, ?, ?, ?)",
            params![playlist_id, url, title, new_pos],
        ).context("Failed to add song to playlist")?;

        Ok(())
    }

    pub fn get_songs(&self, playlist_id: i64) -> anyhow::Result<Vec<PlaylistEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, playlist_id, url, title, position FROM playlist_entries WHERE playlist_id = ? ORDER BY position ASC")?;

        let song_iter = stmt.query_map([playlist_id], |row| {
            Ok(PlaylistEntry {
                id: row.get(0)?,
                playlist_id: row.get(1)?,
                url: row.get(2)?,
                title: row.get(3)?,
                position: row.get(4)?,
            })
        })?;

        let mut songs = Vec::new();
        for song in song_iter {
            songs.push(song?);
        }

        Ok(songs)
    }
}
