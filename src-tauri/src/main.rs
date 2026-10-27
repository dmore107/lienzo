// Lienzo — portapapeles con lienzo y línea de tiempo.
// Todo se guarda localmente en %LOCALAPPDATA%\com.dani.lienzo (nada sale del equipo).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod acta;
mod db;
mod export;
mod portable;
mod rec;
mod transcribe;
mod updater;
mod win;

use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicU64, Ordering},
        Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::Engine;
use db::COLS;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WindowEvent, Wry,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const TRASH_DAYS: i64 = 30;
const THUMB_W: u32 = 440;
const AUTOSTART_ARG: &str = "--autostart";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Item {
    id: String,
    kind: String,
    content: String,
    note: String,
    ts: i64,
    pinned: bool,
    deleted_at: Option<i64>,
    x: Option<f64>,
    y: Option<f64>,
    tone: String,
    pin_order: Option<f64>,
    source: String,
    thumb: Option<String>,
}

struct AppState {
    db: Mutex<Connection>,
    img_dir: PathBuf,
    sess_dir: PathBuf,
    portable: bool,
    paused: AtomicBool,
    keep_open: AtomicBool,
    writing: AtomicBool,
    skip_seq: AtomicU32,
    last_hash: AtomicU64,
    prev_window: AtomicIsize,
    counter: AtomicU64,
    hotkeys: Mutex<Vec<String>>,
}

struct PauseItem(CheckMenuItem<Wry>);
struct AutostartItem(CheckMenuItem<Wry>);
struct RecItem(MenuItem<Wry>);

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// FNV-1a: estable entre versiones, porque la huella se guarda en la base para evitar duplicados.
fn hash_bytes(b: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &x in b {
        h ^= x as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

fn hex(h: u64) -> String {
    format!("{h:016x}")
}

impl AppState {
    fn new_id(&self) -> String {
        format!("{:x}{:04x}", now_ms(), self.counter.fetch_add(1, Ordering::SeqCst) & 0xffff)
    }
    fn thumb_path(&self, name: &str) -> PathBuf {
        let stem = Path::new(name).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        self.img_dir.join("thumbs").join(format!("{stem}.jpg"))
    }
    /// Miniatura JPG de 440 px de ancho para las tarjetas (la original queda intacta para pegar).
    fn make_thumb(&self, name: &str, img: &image::RgbaImage) {
        let (w, h) = img.dimensions();
        if w == 0 || h == 0 {
            return;
        }
        let (tw, th) = if w > THUMB_W { (THUMB_W, ((h as u64 * THUMB_W as u64) / w as u64).max(1) as u32) } else { (w, h) };
        let t = image::imageops::thumbnail(img, tw, th);
        // JPG no tiene transparencia: se mezcla sobre blanco
        let rgb = image::RgbImage::from_fn(tw, th, |x, y| {
            let p = t.get_pixel(x, y).0;
            let a = p[3] as u32;
            let mix = |c: u8| ((c as u32 * a + 255 * (255 - a)) / 255) as u8;
            image::Rgb([mix(p[0]), mix(p[1]), mix(p[2])])
        });
        let _ = rgb.save(self.thumb_path(name));
    }
    fn remove_image(&self, name: &str) {
        let _ = std::fs::remove_file(self.img_dir.join(name));
        let _ = std::fs::remove_file(self.thumb_path(name));
    }
    fn row_to_item(&self, r: &rusqlite::Row) -> rusqlite::Result<Item> {
        let kind: String = r.get(1)?;
        let mut content: String = r.get(2)?;
        let mut thumb = None;
        if kind == "image" {
            let t = self.thumb_path(&content);
            if t.exists() {
                thumb = Some(t.to_string_lossy().into_owned());
            }
            content = self.img_dir.join(&content).to_string_lossy().into_owned();
        }
        Ok(Item {
            id: r.get(0)?,
            kind,
            content,
            note: r.get(3)?,
            ts: r.get(4)?,
            pinned: r.get::<_, i64>(5)? != 0,
            deleted_at: r.get(6)?,
            x: r.get(7)?,
            y: r.get(8)?,
            tone: r.get(9)?,
            pin_order: r.get(10)?,
            source: r.get(11)?,
            thumb,
        })
    }
    fn get(&self, id: &str) -> Option<Item> {
        let db = self.db.lock().ok()?;
        db.query_row(&format!("SELECT {COLS} FROM items WHERE id=?1"), params![id], |r| self.row_to_item(r))
            .optional()
            .ok()
            .flatten()
    }
    fn insert(&self, kind: &str, content: &str, note: &str, hash: &str, source: &str) -> Result<Item, String> {
        let id = self.new_id();
        let ts = now_ms();
        {
            let db = self.db.lock().map_err(|e| e.to_string())?;
            db.execute(
                "INSERT INTO items (id,kind,content,note,ts,pinned,hash,source) VALUES (?1,?2,?3,?4,?5,0,?6,?7)",
                params![id, kind, content, note, ts, hash, source],
            )
            .map_err(|e| e.to_string())?;
        }
        self.get(&id).ok_or_else(|| "no se pudo leer el elemento".into())
    }
    /// Si ya se había copiado lo mismo, lo trae al presente (y lo saca de la papelera) en lugar de duplicarlo.
    fn bump_existing(&self, hash: &str, source: &str) -> Option<Item> {
        let id = {
            let db = self.db.lock().ok()?;
            let id: String = db
                .query_row("SELECT id FROM items WHERE hash=?1 ORDER BY ts DESC LIMIT 1", params![hash], |r| r.get(0))
                .optional()
                .ok()??;
            db.execute(
                "UPDATE items SET ts=?1, deleted_at=NULL, source=CASE WHEN ?2='' THEN source ELSE ?2 END WHERE id=?3",
                params![now_ms(), source, id],
            )
            .ok()?;
            id
        };
        self.get(&id)
    }
    fn save_rgba(&self, w: u32, h: u32, rgba: Vec<u8>, hash: &str, source: &str) -> Result<Item, String> {
        let img = image::RgbaImage::from_raw(w, h, rgba).ok_or("imagen inválida")?;
        let name = format!("{}.png", self.new_id());
        img.save(self.img_dir.join(&name)).map_err(|e| e.to_string())?;
        self.make_thumb(&name, &img);
        self.insert("image", &name, "", hash, source)
    }
}

fn purge_old_trash(state: &AppState) {
    let limit = now_ms() - TRASH_DAYS * 86_400_000;
    let images: Vec<String> = {
        let db = match state.db.lock() { Ok(d) => d, Err(_) => return };
        let mut st = match db.prepare("SELECT content FROM items WHERE kind='image' AND deleted_at IS NOT NULL AND deleted_at < ?1") {
            Ok(s) => s,
            Err(_) => return,
        };
        let v = st.query_map(params![limit], |r| r.get::<_, String>(0)).map(|it| it.flatten().collect()).unwrap_or_default();
        let _ = db.execute("DELETE FROM items WHERE deleted_at IS NOT NULL AND deleted_at < ?1", params![limit]);
        v
    };
    for f in images {
        state.remove_image(&f);
    }
    rec::purge_trash(state, Some(limit));
}

/// Crea en segundo plano las miniaturas de imágenes guardadas antes de la 0.2.
fn backfill_thumbs(app: AppHandle) {
    std::thread::spawn(move || {
        let state = app.state::<AppState>();
        let names: Vec<String> = {
            let db = match state.db.lock() { Ok(d) => d, Err(_) => return };
            let mut st = match db.prepare("SELECT content FROM items WHERE kind='image'") {
                Ok(s) => s,
                Err(_) => return,
            };
            let v = st.query_map([], |r| r.get::<_, String>(0)).map(|it| it.flatten().collect()).unwrap_or_default();
            v
        };
        let mut made = 0;
        for n in names {
            if state.thumb_path(&n).exists() {
                continue;
            }
            if let Ok(img) = image::open(state.img_dir.join(&n)) {
                state.make_thumb(&n, &img.to_rgba8());
                made += 1;
            }
        }
        if made > 0 {
            let _ = app.emit("thumbs-ready", ());
        }
    });
}

/* ===================== Captura del portapapeles ===================== */

fn capture_once(app: &AppHandle) -> Result<(), ()> {
    let state = app.state::<AppState>();
    if win::excluded() {
        return Ok(());
    }
    let source = win::clipboard_source();
    let mut cb = arboard::Clipboard::new().map_err(|_| ())?;

    if let Ok(text) = cb.get_text() {
        let t = text.trim_end_matches(['\r', '\n']);
        if t.trim().is_empty() || t.len() > 2_000_000 {
            return Ok(());
        }
        let h = hash_bytes(t.as_bytes());
        if state.last_hash.swap(h, Ordering::SeqCst) == h {
            return Ok(());
        }
        let hx = hex(h);
        let item = match state.bump_existing(&hx, &source) {
            Some(it) => Ok(it),
            None => state.insert("text", t, "", &hx, &source),
        };
        if let Ok(item) = item {
            let _ = app.emit("item-added", item);
        }
        return Ok(());
    }

    match cb.get_image() {
        Ok(img) => {
            let h = hash_bytes(&img.bytes);
            if state.last_hash.swap(h, Ordering::SeqCst) == h {
                return Ok(());
            }
            let hx = hex(h);
            let item = match state.bump_existing(&hx, &source) {
                Some(it) => Ok(it),
                None => state.save_rgba(img.width as u32, img.height as u32, img.bytes.into_owned(), &hx, &source),
            };
            if let Ok(item) = item {
                let _ = app.emit("item-added", item);
            }
            Ok(())
        }
        Err(arboard::Error::ContentNotAvailable) => Ok(()),
        Err(_) => Err(()),
    }
}

fn start_watcher(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last = win::seq();
        loop {
            std::thread::sleep(Duration::from_millis(350));
            let state = app.state::<AppState>();
            let s = win::seq();
            if s == last {
                continue;
            }
            if state.writing.load(Ordering::SeqCst) {
                continue;
            }
            if s == state.skip_seq.load(Ordering::SeqCst) || state.paused.load(Ordering::SeqCst) {
                last = s;
                continue;
            }
            std::thread::sleep(Duration::from_millis(60));
            for _ in 0..4 {
                if capture_once(&app).is_ok() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(120));
            }
            last = s;
        }
    });
}

/* ===================== Ventana ===================== */

fn show_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let fg = win::foreground();
        let own = w.hwnd().map(|h| h.0 as isize).unwrap_or(-1);
        if fg != 0 && fg != own {
            app.state::<AppState>().prev_window.store(fg, Ordering::SeqCst);
        }
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        let _ = w.emit("shown", ());
    }
}

fn toggle_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_visible().unwrap_or(false) && w.is_focused().unwrap_or(false) {
            let _ = w.hide();
        } else {
            show_window(app);
        }
    }
}

fn set_paused_inner(app: &AppHandle, v: bool) {
    app.state::<AppState>().paused.store(v, Ordering::SeqCst);
    if let Some(p) = app.try_state::<PauseItem>() {
        let _ = p.0.set_checked(v);
    }
    let _ = app.emit("paused", v);
}

fn toggle_autostart(app: &AppHandle) {
    let al = app.autolaunch();
    let on = al.is_enabled().unwrap_or(false);
    let _ = if on { al.disable() } else { al.enable() };
    if let Some(a) = app.try_state::<AutostartItem>() {
        let _ = a.0.set_checked(al.is_enabled().unwrap_or(false));
    }
}

/* ===================== Comandos para la interfaz ===================== */

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    paused: bool,
    keep_open: bool,
    hotkeys: Vec<String>,
    data_dir: String,
    version: String,
    portable: bool,
}

#[tauri::command]
fn status(app: AppHandle, state: State<AppState>) -> Status {
    Status {
        paused: state.paused.load(Ordering::SeqCst),
        keep_open: state.keep_open.load(Ordering::SeqCst),
        hotkeys: state.hotkeys.lock().map(|h| h.clone()).unwrap_or_default(),
        data_dir: state.img_dir.parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default(),
        version: app.package_info().version.to_string(),
        portable: state.portable,
    }
}

#[tauri::command]
fn list_items(state: State<AppState>) -> Result<Vec<Item>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut st = db.prepare(&format!("SELECT {COLS} FROM items ORDER BY ts")).map_err(|e| e.to_string())?;
    let rows = st.query_map([], |r| state.row_to_item(r)).map_err(|e| e.to_string())?;
    Ok(rows.flatten().collect())
}

type Patch = serde_json::Map<String, serde_json::Value>;

fn apply_patch(db: &Connection, id: &str, patch: Patch) -> rusqlite::Result<()> {
    for (k, v) in patch {
        let col = match k.as_str() {
            "content" => "content",
            "note" => "note",
            "pinned" => "pinned",
            "deletedAt" => "deleted_at",
            "x" => "x",
            "y" => "y",
            "tone" => "tone",
            "pinOrder" => "pin_order",
            _ => continue,
        };
        if col == "content" {
            // Solo los textos se pueden editar; la huella sigue al contenido nuevo
            let s = v.as_str().unwrap_or_default();
            db.execute("UPDATE items SET content=?1, hash=?2 WHERE id=?3 AND kind='text'", params![s, hex(hash_bytes(s.as_bytes())), id])?;
            continue;
        }
        let sql = format!("UPDATE items SET {col}=?1 WHERE id=?2");
        match v {
            serde_json::Value::Null => db.execute(&sql, params![Option::<i64>::None, id]),
            serde_json::Value::Bool(b) => db.execute(&sql, params![b as i64, id]),
            serde_json::Value::Number(n) if col == "x" || col == "y" || col == "pin_order" => db.execute(&sql, params![n.as_f64(), id]),
            serde_json::Value::Number(n) => db.execute(&sql, params![n.as_i64(), id]),
            serde_json::Value::String(s) => db.execute(&sql, params![s, id]),
            _ => continue,
        }?;
    }
    Ok(())
}

#[tauri::command]
fn update_item(state: State<AppState>, id: String, patch: Patch) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    apply_patch(&db, &id, patch).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct ItemPatch {
    id: String,
    patch: Patch,
}

/// Varios cambios en una sola transacción (p. ej. al pasar todo el tablero a modo libre).
#[tauri::command]
fn update_items(state: State<AppState>, patches: Vec<ItemPatch>) -> Result<(), String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    let tx = db.transaction().map_err(|e| e.to_string())?;
    for p in patches {
        apply_patch(&tx, &p.id, p.patch).map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())
}

#[tauri::command]
fn create_text(state: State<AppState>, content: String) -> Result<Item, String> {
    state.insert("text", &content, "", &hex(hash_bytes(content.as_bytes())), "")
}

#[tauri::command]
fn create_image(state: State<AppState>, data_url: String) -> Result<Item, String> {
    let b64 = data_url.split_once(',').map(|x| x.1).unwrap_or(&data_url);
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?.to_rgba8();
    let (w, h) = img.dimensions();
    let raw = img.into_raw();
    let hx = hex(hash_bytes(&raw));
    state.save_rgba(w, h, raw, &hx, "")
}

#[tauri::command]
fn delete_forever(state: State<AppState>, id: String) -> Result<(), String> {
    let image: Option<String> = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let name = db
            .query_row("SELECT content FROM items WHERE id=?1 AND kind='image'", params![id], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        db.execute("DELETE FROM items WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
        name
    };
    if let Some(name) = image {
        state.remove_image(&name);
    }
    Ok(())
}

#[tauri::command]
fn empty_trash(state: State<AppState>) -> Result<usize, String> {
    let (images, n) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut st = db
            .prepare("SELECT content FROM items WHERE kind='image' AND deleted_at IS NOT NULL")
            .map_err(|e| e.to_string())?;
        let v: Vec<String> = st.query_map([], |r| r.get(0)).map_err(|e| e.to_string())?.flatten().collect();
        let n = db.execute("DELETE FROM items WHERE deleted_at IS NOT NULL", []).map_err(|e| e.to_string())?;
        (v, n)
    };
    for f in images {
        state.remove_image(&f);
    }
    Ok(n + rec::purge_trash(&state, None))
}

#[tauri::command]
fn copy_item(app: AppHandle, state: State<AppState>, id: String, paste: bool) -> Result<(), String> {
    let it = state.get(&id).ok_or("no existe")?;
    state.writing.store(true, Ordering::SeqCst);
    let res = (|| -> Result<(), String> {
        let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        if it.kind == "text" {
            state.last_hash.store(hash_bytes(it.content.as_bytes()), Ordering::SeqCst);
            cb.set_text(it.content.clone()).map_err(|e| e.to_string())?;
        } else {
            let img = image::open(&it.content).map_err(|e| e.to_string())?.to_rgba8();
            let (w, h) = img.dimensions();
            let raw = img.into_raw();
            state.last_hash.store(hash_bytes(&raw), Ordering::SeqCst);
            cb.set_image(arboard::ImageData { width: w as usize, height: h as usize, bytes: raw.into() })
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    })();
    state.skip_seq.store(win::seq(), Ordering::SeqCst);
    state.writing.store(false, Ordering::SeqCst);
    res?;

    if paste {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.hide();
        }
        let prev = state.prev_window.load(Ordering::SeqCst);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(90));
            win::focus(prev);
            std::thread::sleep(Duration::from_millis(140));
            win::send_ctrl_v();
        });
    }
    Ok(())
}

/// Copia texto (p. ej. una transcripción) sin que Lienzo lo vuelva a capturar como copia nueva.
#[tauri::command]
fn copy_text(state: State<AppState>, text: String) -> Result<(), String> {
    state.writing.store(true, Ordering::SeqCst);
    state.last_hash.store(hash_bytes(text.as_bytes()), Ordering::SeqCst);
    let res = arboard::Clipboard::new().and_then(|mut cb| cb.set_text(text)).map_err(|e| e.to_string());
    state.skip_seq.store(win::seq(), Ordering::SeqCst);
    state.writing.store(false, Ordering::SeqCst);
    res
}

#[tauri::command]
fn set_paused(app: AppHandle, value: bool) {
    set_paused_inner(&app, value);
}

#[tauri::command]
fn set_keep_open(state: State<AppState>, value: bool) {
    state.keep_open.store(value, Ordering::SeqCst);
}

#[derive(Serialize)]
struct SysColors {
    accent: String,
    dark: bool,
}

#[tauri::command]
fn system_colors() -> SysColors {
    let (accent, dark) = win::system_colors();
    SysColors { accent, dark }
}

/// Cambia entre la vista compacta (tipo widget, abajo a la derecha) y la completa (centrada).
#[tauri::command]
fn set_compact(app: AppHandle, value: bool) {
    use tauri::{LogicalPosition, LogicalSize};
    let Some(w) = app.get_webview_window("main") else { return };
    let (cw, ch) = if value { (400.0, 560.0) } else { (1180.0, 740.0) };
    let _ = w.set_size(LogicalSize::new(cw, ch));
    if value {
        if let Ok(Some(m)) = w.current_monitor() {
            let sf = m.scale_factor();
            let wa = m.work_area();
            let x = (wa.position.x as f64 + wa.size.width as f64) / sf - cw - 16.0;
            let y = (wa.position.y as f64 + wa.size.height as f64) / sf - ch - 16.0;
            let _ = w.set_position(LogicalPosition::new(x, y));
        }
    } else {
        let _ = w.center();
    }
}

#[tauri::command]
fn hide_window(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}

/* ===================== Arranque ===================== */

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| show_window(app)))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![AUTOSTART_ARG])))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, sc, ev| {
                    if ev.state() == ShortcutState::Pressed {
                        if *sc == rec::shortcut() {
                            rec::toggle(app);
                        } else {
                            toggle_window(app);
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            portable::cleanup_old();
            let portable_dir = portable::data_dir();
            let base = match &portable_dir {
                Some(d) => d.clone(),
                None => app.path().app_local_data_dir().expect("sin carpeta de datos"),
            };
            // La ventana se crea aquí para que, en modo portable, sus datos también queden en "datos"
            let wcfg = app.config().app.windows.iter().find(|w| w.label == "main").cloned().ok_or("falta la ventana principal")?;
            let mut wb = tauri::WebviewWindowBuilder::from_config(app.handle(), &wcfg)?;
            if let Some(d) = &portable_dir {
                wb = wb.data_directory(d.join("webview"));
            }
            wb.build()?;
            let _ = app.asset_protocol_scope().allow_directory(&base, true);
            let img_dir = base.join("img");
            let sess_dir = base.join("sesiones");
            std::fs::create_dir_all(img_dir.join("thumbs"))?;
            std::fs::create_dir_all(&sess_dir)?;
            let conn = match db::open(&base) {
                Ok(c) => c,
                Err(e) => {
                    win::alert(&format!("Lienzo no pudo abrir su base de datos.\n\n{e}"));
                    return Err(e.into());
                }
            };
            app.manage(AppState {
                db: Mutex::new(conn),
                img_dir,
                sess_dir,
                portable: portable_dir.is_some(),
                paused: AtomicBool::new(false),
                keep_open: AtomicBool::new(false),
                writing: AtomicBool::new(false),
                skip_seq: AtomicU32::new(0),
                last_hash: AtomicU64::new(0),
                prev_window: AtomicIsize::new(0),
                counter: AtomicU64::new(0),
                hotkeys: Mutex::new(vec![]),
            });
            app.manage(rec::Recorder::default());
            app.manage(transcribe::Transcriber::default());
            purge_old_trash(&app.state::<AppState>());
            rec::recover(&app.state::<AppState>());
            transcribe::recover(&app.state::<AppState>());

            // Atajos globales: Win+V (si Windows lo deja libre) y Alt+V
            let mut ok = vec![];
            for (sc, label) in [
                (Shortcut::new(Some(Modifiers::SUPER), Code::KeyV), "Win+V"),
                (Shortcut::new(Some(Modifiers::ALT), Code::KeyV), "Alt+V"),
            ] {
                if app.global_shortcut().register(sc).is_ok() {
                    ok.push(label.to_string());
                }
            }
            *app.state::<AppState>().hotkeys.lock().unwrap() = ok.clone();
            let _ = app.global_shortcut().register(rec::shortcut());

            // Bandeja del sistema
            let version = app.package_info().version.to_string();
            let label = format!("Abrir  ({})", ok.first().cloned().unwrap_or_else(|| "sin atajo".into()));
            let open = MenuItem::with_id(app, "open", label, true, None::<&str>)?;
            let record = MenuItem::with_id(app, "record", "Grabar reunión  (Alt+Shift+R)", true, None::<&str>)?;
            let pause = CheckMenuItem::with_id(app, "pause", "Pausar captura", true, false, None::<&str>)?;
            let auto_on = app.autolaunch().is_enabled().unwrap_or(false);
            let auto = CheckMenuItem::with_id(app, "autostart", "Iniciar con Windows", true, auto_on, None::<&str>)?;
            let update = MenuItem::with_id(app, "update", format!("Buscar actualizaciones  (v{version})"), true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
            // La portable no toca el registro: sin "Iniciar con Windows"
            let mut entries: Vec<&dyn tauri::menu::IsMenuItem<Wry>> = vec![&open, &record, &pause];
            if portable_dir.is_none() {
                entries.push(&auto);
            }
            entries.extend([&update as &dyn tauri::menu::IsMenuItem<Wry>, &sep, &quit]);
            let menu = Menu::with_items(app, &entries)?;
            app.manage(PauseItem(pause.clone()));
            app.manage(AutostartItem(auto.clone()));
            app.manage(RecItem(record.clone()));
            TrayIconBuilder::with_id("tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip(format!("Lienzo {version} — portapapeles"))
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, e| match e.id.as_ref() {
                    "open" => show_window(app),
                    "pause" => {
                        let v = !app.state::<AppState>().paused.load(Ordering::SeqCst);
                        set_paused_inner(app, v);
                    }
                    "record" => rec::toggle(app),
                    "autostart" => toggle_autostart(app),
                    "update" => {
                        show_window(app);
                        let _ = app.emit("check-update", ());
                    }
                    "quit" => {
                        let _ = rec::stop(app);
                        app.exit(0)
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, e| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // Al iniciar con Windows queda solo en la bandeja; abierto a mano, se muestra.
            let autostarted = std::env::args().any(|a| a == AUTOSTART_ARG);
            if let Some(w) = app.get_webview_window("main") {
                // Efecto de vidrio de Windows detrás de la ventana
                #[cfg(windows)]
                {
                    let _ = window_vibrancy::apply_acrylic(&w, Some((0, 0, 0, 0)))
                        .or_else(|_| window_vibrancy::apply_mica(&w, None));
                }
                if !autostarted {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }

            start_watcher(app.handle().clone());
            backfill_thumbs(app.handle().clone());
            transcribe::start_worker(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
            }
            WindowEvent::Focused(false) => {
                let app = window.app_handle();
                if !app.state::<AppState>().keep_open.load(Ordering::SeqCst) {
                    let _ = window.hide();
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            status,
            list_items,
            update_item,
            update_items,
            create_text,
            create_image,
            delete_forever,
            empty_trash,
            copy_item,
            copy_text,
            set_paused,
            set_keep_open,
            system_colors,
            set_compact,
            hide_window,
            rec::rec_start,
            rec::rec_stop,
            rec::rec_pause,
            rec::rec_mute,
            rec::rec_status,
            rec::list_sessions,
            rec::update_session,
            rec::delete_session,
            rec::open_session_folder,
            transcribe::transcribe_setup,
            transcribe::set_transcribe_prefs,
            transcribe::download_model,
            transcribe::cancel_download,
            transcribe::delete_model,
            transcribe::transcribe_session,
            transcribe::cancel_transcription,
            transcribe::get_transcript,
            acta::gemini_status,
            acta::set_gemini_key,
            acta::delete_gemini_key,
            acta::generate_acta,
            export::export_session,
            updater::check_update,
            updater::install_update
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar Lienzo");
}
