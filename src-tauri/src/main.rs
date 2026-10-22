// Lienzo — portapapeles con lienzo y línea de tiempo.
// Todo se guarda localmente en %LOCALAPPDATA%\com.dani.lienzo (nada sale del equipo).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicU64, Ordering},
        Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::Engine;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WindowEvent, Wry,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const TRASH_DAYS: i64 = 30;

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
}

struct AppState {
    db: Mutex<Connection>,
    img_dir: PathBuf,
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

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn hash_bytes(b: &[u8]) -> u64 {
    let mut h = DefaultHasher::new();
    b.hash(&mut h);
    h.finish()
}

impl AppState {
    fn new_id(&self) -> String {
        format!("{:x}{:04x}", now_ms(), self.counter.fetch_add(1, Ordering::SeqCst) & 0xffff)
    }
    fn row_to_item(&self, r: &rusqlite::Row) -> rusqlite::Result<Item> {
        let kind: String = r.get(1)?;
        let mut content: String = r.get(2)?;
        if kind == "image" {
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
        })
    }
    fn get(&self, id: &str) -> Option<Item> {
        let db = self.db.lock().ok()?;
        db.query_row(
            "SELECT id,kind,content,note,ts,pinned,deleted_at,x,y,tone,pin_order FROM items WHERE id=?1",
            params![id],
            |r| self.row_to_item(r),
        )
        .optional()
        .ok()
        .flatten()
    }
    fn insert(&self, kind: &str, content: &str, note: &str) -> Result<Item, String> {
        let id = self.new_id();
        let ts = now_ms();
        {
            let db = self.db.lock().map_err(|e| e.to_string())?;
            db.execute(
                "INSERT INTO items (id,kind,content,note,ts,pinned) VALUES (?1,?2,?3,?4,?5,0)",
                params![id, kind, content, note, ts],
            )
            .map_err(|e| e.to_string())?;
        }
        self.get(&id).ok_or_else(|| "no se pudo leer el elemento".into())
    }
    fn save_rgba(&self, w: u32, h: u32, rgba: Vec<u8>) -> Result<Item, String> {
        let img = image::RgbaImage::from_raw(w, h, rgba).ok_or("imagen inválida")?;
        let name = format!("{}.png", self.new_id());
        img.save(self.img_dir.join(&name)).map_err(|e| e.to_string())?;
        self.insert("image", &name, "")
    }
}

/* ===================== Base de datos ===================== */

fn open_db(dir: &PathBuf) -> Connection {
    let conn = Connection::open(dir.join("lienzo.db")).expect("no se pudo abrir la base de datos");
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS items(
           id TEXT PRIMARY KEY, kind TEXT NOT NULL, content TEXT NOT NULL,
           note TEXT NOT NULL DEFAULT '', ts INTEGER NOT NULL, pinned INTEGER NOT NULL DEFAULT 0,
           deleted_at INTEGER, x REAL, y REAL);
         CREATE INDEX IF NOT EXISTS idx_items_ts ON items(ts);",
    )
    .expect("no se pudo crear la tabla");
    // Columnas nuevas (versiones anteriores no las tenían)
    let _ = conn.execute("ALTER TABLE items ADD COLUMN tone TEXT NOT NULL DEFAULT ''", []);
    let _ = conn.execute("ALTER TABLE items ADD COLUMN pin_order REAL", []);
    conn
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
        let _ = std::fs::remove_file(state.img_dir.join(f));
    }
}

/* ===================== Windows: portapapeles, foco y pegado ===================== */

#[cfg(windows)]
mod win {
    use windows_sys::Win32::{
        System::DataExchange::*,
        System::Memory::{GlobalLock, GlobalUnlock},
        UI::Input::KeyboardAndMouse::*,
        UI::WindowsAndMessaging::{GetForegroundWindow, SetForegroundWindow},
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn seq() -> u32 {
        unsafe { GetClipboardSequenceNumber() }
    }

    /// Respeta a los gestores de contraseñas y apps que piden no guardar su contenido.
    pub fn excluded() -> bool {
        unsafe {
            let ex = RegisterClipboardFormatW(wide("ExcludeClipboardContentFromMonitorProcessing").as_ptr());
            if ex != 0 && IsClipboardFormatAvailable(ex) != 0 {
                return true;
            }
            let hist = RegisterClipboardFormatW(wide("CanIncludeInClipboardHistory").as_ptr());
            if hist != 0 && IsClipboardFormatAvailable(hist) != 0 {
                let mut allowed = true;
                if OpenClipboard(std::ptr::null_mut()) != 0 {
                    let h = GetClipboardData(hist);
                    if !h.is_null() {
                        let p = GlobalLock(h) as *const u32;
                        if !p.is_null() {
                            allowed = *p != 0;
                            GlobalUnlock(h);
                        }
                    }
                    CloseClipboard();
                }
                return !allowed;
            }
            false
        }
    }

    pub fn foreground() -> isize {
        unsafe { GetForegroundWindow() as isize }
    }

    pub fn focus(h: isize) {
        if h != 0 {
            unsafe { SetForegroundWindow(h as _) };
        }
    }

    fn reg_dword(sub: &str, name: &str) -> Option<u32> {
        use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
        let (sub, name) = (wide(sub), wide(name));
        let mut data: u32 = 0;
        let mut size: u32 = 4;
        let r = unsafe {
            RegGetValueW(HKEY_CURRENT_USER, sub.as_ptr(), name.as_ptr(), RRF_RT_REG_DWORD, std::ptr::null_mut(), &mut data as *mut u32 as *mut _, &mut size)
        };
        if r == 0 { Some(data) } else { None }
    }

    /// Color de acento de Windows (#rrggbb) y si las apps usan modo oscuro.
    pub fn system_colors() -> (String, bool) {
        let accent = reg_dword("Software\\Microsoft\\Windows\\DWM", "AccentColor")
            .map(|v| (v & 0xff, (v >> 8) & 0xff, (v >> 16) & 0xff)) // ABGR
            .or_else(|| reg_dword("Software\\Microsoft\\Windows\\DWM", "ColorizationColor").map(|v| ((v >> 16) & 0xff, (v >> 8) & 0xff, v & 0xff))) // ARGB
            .unwrap_or((0, 120, 212));
        let dark = reg_dword("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize", "AppsUseLightTheme").map(|v| v == 0).unwrap_or(false);
        (format!("#{:02x}{:02x}{:02x}", accent.0, accent.1, accent.2), dark)
    }

    pub fn send_ctrl_v() {
        fn key(vk: u16, up: bool) -> INPUT {
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT { wVk: vk, wScan: 0, dwFlags: if up { KEYEVENTF_KEYUP } else { 0 }, time: 0, dwExtraInfo: 0 },
                },
            }
        }
        // Suelta Alt/Shift por si siguen presionadas por el atajo
        let inputs = [
            key(VK_MENU, true),
            key(VK_SHIFT, true),
            key(VK_CONTROL, false),
            key(0x56, false),
            key(0x56, true),
            key(VK_CONTROL, true),
        ];
        unsafe { SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32) };
    }
}

#[cfg(not(windows))]
mod win {
    pub fn seq() -> u32 { 0 }
    pub fn excluded() -> bool { false }
    pub fn foreground() -> isize { 0 }
    pub fn focus(_: isize) {}
    pub fn send_ctrl_v() {}
    pub fn system_colors() -> (String, bool) { ("#0078d4".into(), false) }
}

/* ===================== Captura del portapapeles ===================== */

fn capture_once(app: &AppHandle) -> Result<(), ()> {
    let state = app.state::<AppState>();
    if win::excluded() {
        return Ok(());
    }
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
        if let Ok(item) = state.insert("text", t, "") {
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
            if let Ok(item) = state.save_rgba(img.width as u32, img.height as u32, img.bytes.into_owned()) {
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

/* ===================== Comandos para la interfaz ===================== */

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    paused: bool,
    keep_open: bool,
    hotkeys: Vec<String>,
    data_dir: String,
}

#[tauri::command]
fn status(state: State<AppState>) -> Status {
    Status {
        paused: state.paused.load(Ordering::SeqCst),
        keep_open: state.keep_open.load(Ordering::SeqCst),
        hotkeys: state.hotkeys.lock().map(|h| h.clone()).unwrap_or_default(),
        data_dir: state.img_dir.parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default(),
    }
}

#[tauri::command]
fn list_items(state: State<AppState>) -> Result<Vec<Item>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut st = db
        .prepare("SELECT id,kind,content,note,ts,pinned,deleted_at,x,y,tone,pin_order FROM items ORDER BY ts")
        .map_err(|e| e.to_string())?;
    let rows = st.query_map([], |r| state.row_to_item(r)).map_err(|e| e.to_string())?;
    Ok(rows.flatten().collect())
}

#[tauri::command]
fn update_item(state: State<AppState>, id: String, patch: serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
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
            // Solo los textos se pueden editar
            let s = v.as_str().unwrap_or_default();
            db.execute("UPDATE items SET content=?1 WHERE id=?2 AND kind='text'", params![s, id]).map_err(|e| e.to_string())?;
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
        }
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn create_text(state: State<AppState>, content: String) -> Result<Item, String> {
    state.insert("text", &content, "")
}

#[tauri::command]
fn create_image(state: State<AppState>, data_url: String) -> Result<Item, String> {
    let b64 = data_url.split_once(',').map(|x| x.1).unwrap_or(&data_url);
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?.to_rgba8();
    let (w, h) = img.dimensions();
    state.save_rgba(w, h, img.into_raw())
}

#[tauri::command]
fn delete_forever(state: State<AppState>, id: String) -> Result<(), String> {
    let item = state.get(&id);
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute("DELETE FROM items WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
    }
    if let Some(it) = item {
        if it.kind == "image" {
            let _ = std::fs::remove_file(&it.content);
        }
    }
    Ok(())
}

#[tauri::command]
fn empty_trash(state: State<AppState>) -> Result<usize, String> {
    let images: Vec<String> = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut st = db
            .prepare("SELECT content FROM items WHERE kind='image' AND deleted_at IS NOT NULL")
            .map_err(|e| e.to_string())?;
        let v: Vec<String> = st.query_map([], |r| r.get(0)).map_err(|e| e.to_string())?.flatten().collect();
        v
    };
    let n = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute("DELETE FROM items WHERE deleted_at IS NOT NULL", []).map_err(|e| e.to_string())?
    };
    for f in images {
        let _ = std::fs::remove_file(state.img_dir.join(f));
    }
    Ok(n)
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
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _sc, ev| {
                    if ev.state() == ShortcutState::Pressed {
                        toggle_window(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let base = app.path().app_local_data_dir().expect("sin carpeta de datos");
            let img_dir = base.join("img");
            std::fs::create_dir_all(&img_dir)?;
            let conn = open_db(&base);
            app.manage(AppState {
                db: Mutex::new(conn),
                img_dir,
                paused: AtomicBool::new(false),
                keep_open: AtomicBool::new(false),
                writing: AtomicBool::new(false),
                skip_seq: AtomicU32::new(0),
                last_hash: AtomicU64::new(0),
                prev_window: AtomicIsize::new(0),
                counter: AtomicU64::new(0),
                hotkeys: Mutex::new(vec![]),
            });
            purge_old_trash(&app.state::<AppState>());

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

            // Bandeja del sistema
            let label = format!("Abrir  ({})", ok.first().cloned().unwrap_or_else(|| "sin atajo".into()));
            let open = MenuItem::with_id(app, "open", label, true, None::<&str>)?;
            let pause = CheckMenuItem::with_id(app, "pause", "Pausar captura", true, false, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &pause, &quit])?;
            app.manage(PauseItem(pause.clone()));
            TrayIconBuilder::with_id("tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Lienzo — portapapeles")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, e| match e.id.as_ref() {
                    "open" => show_window(app),
                    "pause" => {
                        let v = !app.state::<AppState>().paused.load(Ordering::SeqCst);
                        set_paused_inner(app, v);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, e| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // Efecto de vidrio de Windows detrás de la ventana
            if let Some(w) = app.get_webview_window("main") {
                #[cfg(windows)]
                {
                    let _ = window_vibrancy::apply_acrylic(&w, Some((0, 0, 0, 0)))
                        .or_else(|_| window_vibrancy::apply_mica(&w, None));
                }
                let _ = w.set_focus();
            }

            start_watcher(app.handle().clone());
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
            create_text,
            create_image,
            delete_forever,
            empty_trash,
            copy_item,
            set_paused,
            set_keep_open,
            system_colors,
            set_compact,
            hide_window
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar Lienzo");
}
