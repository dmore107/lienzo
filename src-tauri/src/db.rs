// Base de datos local (SQLite) y sus migraciones.
use std::path::Path;

use rusqlite::{params, Connection};

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
    Ok(())
}
