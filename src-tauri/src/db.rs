// Base de datos local (SQLite) y sus migraciones.
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

/// Columnas que se leen de cada elemento, en el orden que espera `AppState::row_to_item`.
pub const COLS: &str = "id,kind,content,note,ts,pinned,deleted_at,x,y,tone,pin_order,source";

pub fn open(dir: &Path) -> Result<Connection, String> {
    let conn = Connection::open(dir.join("lienzo.db")).map_err(|e| e.to_string())?;
    migrate(&conn).map_err(|e| e.to_string())?;
    Ok(conn)
}

fn has_column(conn: &Connection, col: &str) -> rusqlite::Result<bool> {
    let mut st = conn.prepare("SELECT 1 FROM pragma_table_info('items') WHERE name=?1")?;
    st.exists(params![col])
}

/// Cada migración corre una sola vez; la versión queda en PRAGMA user_version.
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    let v: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;

    // 1: esquema original. Las bases de la 0.1 ya pueden tener tone/pin_order.
    if v < 1 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS items(
               id TEXT PRIMARY KEY, kind TEXT NOT NULL, content TEXT NOT NULL,
               note TEXT NOT NULL DEFAULT '', ts INTEGER NOT NULL, pinned INTEGER NOT NULL DEFAULT 0,
               deleted_at INTEGER, x REAL, y REAL);
             CREATE INDEX IF NOT EXISTS idx_items_ts ON items(ts);",
        )?;
        for (col, ddl) in [("tone", "tone TEXT NOT NULL DEFAULT ''"), ("pin_order", "pin_order REAL")] {
            if !has_column(conn, col)? {
                conn.execute_batch(&format!("ALTER TABLE items ADD COLUMN {ddl}"))?;
            }
        }
        conn.execute_batch("PRAGMA user_version=1;")?;
    }

    // 2: huella del contenido (evita duplicados) y app de origen.
    if v < 2 {
        conn.execute_batch(
            "BEGIN;
             ALTER TABLE items ADD COLUMN hash TEXT;
             ALTER TABLE items ADD COLUMN source TEXT NOT NULL DEFAULT '';
             CREATE INDEX IF NOT EXISTS idx_items_hash ON items(hash);
             PRAGMA user_version=2;
             COMMIT;",
        )?;
    }

    // 3: grabaciones de reuniones y capacitaciones (el audio vive en sesiones/<id>/).
    if v < 3 {
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE IF NOT EXISTS sessions(
               id TEXT PRIMARY KEY, kind TEXT NOT NULL, title TEXT NOT NULL DEFAULT '',
               ts INTEGER NOT NULL, ended_at INTEGER, duration_ms INTEGER NOT NULL DEFAULT 0,
               status TEXT NOT NULL, mic INTEGER NOT NULL DEFAULT 1, system INTEGER NOT NULL DEFAULT 1,
               deleted_at INTEGER, x REAL, y REAL, tone TEXT NOT NULL DEFAULT '');
             CREATE INDEX IF NOT EXISTS idx_sessions_ts ON sessions(ts);
             PRAGMA user_version=3;
             COMMIT;",
        )?;
    }

    // 4: transcripción de las grabaciones y ajustes generales.
    if v < 4 {
        conn.execute_batch(
            "BEGIN;
             ALTER TABLE sessions ADD COLUMN transcript TEXT NOT NULL DEFAULT '';
             ALTER TABLE sessions ADD COLUMN progress REAL NOT NULL DEFAULT 0;
             ALTER TABLE sessions ADD COLUMN error TEXT NOT NULL DEFAULT '';
             CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             PRAGMA user_version=4;
             COMMIT;",
        )?;
    }

    // 5: acta de la reunión o capacitación (JSON generado con Gemini).
    if v < 5 {
        conn.execute_batch(
            "BEGIN;
             ALTER TABLE sessions ADD COLUMN acta TEXT NOT NULL DEFAULT '';
             PRAGMA user_version=5;
             COMMIT;",
        )?;
    }

    // 6: bitácora personal (clics con captura y texto; las imágenes viven en bitacora/) e informes del día.
    if v < 6 {
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE IF NOT EXISTS bitacora(
               id TEXT PRIMARY KEY, ts INTEGER NOT NULL, app TEXT NOT NULL, title TEXT NOT NULL DEFAULT '',
               target TEXT NOT NULL DEFAULT '', image TEXT NOT NULL DEFAULT '', x REAL, y REAL,
               ocr TEXT NOT NULL DEFAULT '');
             CREATE INDEX IF NOT EXISTS idx_bitacora_ts ON bitacora(ts);
             CREATE TABLE IF NOT EXISTS informes(day INTEGER PRIMARY KEY, json TEXT NOT NULL, ts INTEGER NOT NULL);
             PRAGMA user_version=6;
             COMMIT;",
        )?;
    }
    Ok(())
}

pub fn setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row("SELECT value FROM settings WHERE key=?1", params![key], |r| r.get(0)).optional().ok().flatten()
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, value],
    )
    .map(|_| ())
}
