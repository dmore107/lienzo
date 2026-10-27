// Bitácora personal: al hacer clic en las apps que elijas, guarda una captura de esa ventana,
// el nombre de lo que tocaste y el texto que se ve (OCR de Windows, sin internet).
//
// Sirve para revisar después qué hiciste y encontrar un error. Todo queda en este equipo,
// no registra el teclado, no captura campos de contraseña y se borra solo pasados N días.
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use rusqlite::params;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

use crate::{db, now_ms, AppState};

/// Gestores de contraseñas: nunca se capturan aunque estén en la lista.
const NEVER: [&str; 7] = ["keepass", "keepassxc", "1password", "bitwarden", "lastpass", "dashlane", "lienzo"];
/// Entre dos capturas de la misma ventana; los clics intermedios reutilizan la última.
const MIN_GAP: Duration = Duration::from_millis(1500);
/// Espera tras el clic para que la captura muestre el resultado de la acción.
const SETTLE: Duration = Duration::from_millis(350);
const MAX_W: u32 = 1600;

pub fn shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyB)
}

pub struct Click {
    x: i32,
    y: i32,
    at: i64,
}

#[derive(Default)]
pub struct Bitacora {
    on: AtomicBool,
    started: AtomicBool,
    filter: Mutex<Filter>,
}

#[derive(Default, Clone)]
struct Filter {
    apps: Vec<String>,
    title: String,
}

fn dir(state: &AppState) -> PathBuf {
    state.sess_dir.with_file_name("bitacora")
}

/* ===================== Preferencias ===================== */

struct Prefs {
    on: bool,
    apps: Vec<String>,
    title: String,
    days: i64,
}

fn prefs(state: &AppState) -> Prefs {
    let db = state.db.lock();
    let get = |k: &str, d: &str| db.as_ref().ok().and_then(|c| db::setting(c, k)).unwrap_or_else(|| d.to_string());
    Prefs {
        on: get("bit_on", "0") == "1",
        apps: get("bit_apps", "").split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect(),
        title: get("bit_title", "").trim().to_string(),
        days: get("bit_days", "15").parse().unwrap_or(15),
    }
}

/// Al abrir Lienzo: aplica los ajustes guardados y borra lo vencido.
pub fn init(app: &AppHandle) {
    let state = app.state::<AppState>();
    let p = prefs(&state);
    purge(&state, p.days);
    let b = app.state::<Bitacora>();
    if let Ok(mut f) = b.filter.lock() {
        *f = Filter { apps: p.apps.clone(), title: p.title.to_lowercase() };
    }
    if p.on && !p.apps.is_empty() {
        set_on(app, true);
    }
}

fn set_on(app: &AppHandle, on: bool) {
    let b = app.state::<Bitacora>();
    b.on.store(on, Ordering::SeqCst);
    if on && !b.started.swap(true, Ordering::SeqCst) {
        start(app.clone());
    }
    if let Some(i) = app.try_state::<crate::BitItem>() {
        let _ = i.0.set_checked(on);
    }
    let _ = app.emit("bitacora-state", on);
}

/// Atajo y bandeja: activa o pausa la bitácora (si ya elegiste apps).
pub fn toggle(app: &AppHandle) {
    let state = app.state::<AppState>();
    let p = prefs(&state);
    let on = !app.state::<Bitacora>().on.load(Ordering::SeqCst);
    if on && p.apps.is_empty() {
        crate::show_window(app);
        let _ = app.emit("bitacora-setup", ());
        return;
    }
    if let Ok(db) = state.db.lock() {
        let _ = db::set_setting(&db, "bit_on", if on { "1" } else { "0" });
    }
    set_on(app, on);
}

/// Borra lo anterior a `days` días, con sus capturas.
fn purge(state: &AppState, days: i64) {
    let limit = now_ms() - days.max(1) * 86_400_000;
    let images: Vec<String> = {
        let Ok(db) = state.db.lock() else { return };
        let v = db
            .prepare("SELECT DISTINCT image FROM bitacora WHERE ts < ?1 AND image != ''")
            .and_then(|mut st| st.query_map(params![limit], |r| r.get::<_, String>(0)).map(|it| it.flatten().collect()))
            .unwrap_or_default();
        let _ = db.execute("DELETE FROM bitacora WHERE ts < ?1", params![limit]);
        v
    };
    remove_unused(state, images);
}

/// Elimina las capturas que ya ninguna fila usa (varios clics pueden compartir una).
fn remove_unused(state: &AppState, images: Vec<String>) {
    let d = dir(state);
    for img in images {
        let used = state
            .db
            .lock()
            .ok()
            .and_then(|db| db.query_row("SELECT 1 FROM bitacora WHERE image=?1 LIMIT 1", params![img], |_| Ok(())).ok())
            .is_some();
        if !used {
            let _ = std::fs::remove_file(d.join(&img));
        }
    }
}

/* ===================== Captura ===================== */

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    id: String,
    ts: i64,
    app: String,
    title: String,
    target: String,
    image: String,
    x: Option<f64>,
    y: Option<f64>,
    ocr: String,
}

/// Última captura guardada, para reutilizarla en clics seguidos sobre la misma ventana.
struct Last {
    hwnd: isize,
    at: Instant,
    image: String,
    sig: Vec<u8>,
    rect: (i32, i32, i32, i32),
}

/// Huella diminuta (32×18 en grises) para saber si la ventana cambió de verdad.
fn signature(s: &sys::Shot) -> Vec<u8> {
    let (cw, ch) = (32u32, 18u32);
    let mut out = Vec::with_capacity((cw * ch) as usize);
    for j in 0..ch {
        for i in 0..cw {
            let x = (i * s.w / cw) as usize;
            let y = (j * s.h / ch) as usize;
            let p = (y * s.w as usize + x) * 4;
            let px = &s.bgra[p..p + 3];
            out.push(((px[0] as u32 + px[1] as u32 * 2 + px[2] as u32) / 4) as u8);
        }
    }
    out
}

fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).map(|(x, y)| (*x as i32 - *y as i32).unsigned_abs()).sum::<u32>() / (a.len() as u32).max(1) < 3
}

/// Reduce una imagen BGRA para que su lado mayor no pase de `max_side`.
fn shrink(bgra: &[u8], w: u32, h: u32, max_w: u32, max_side: u32) -> (Vec<u8>, u32, u32) {
    let k = (max_w as f64 / w as f64).min(max_side as f64 / w.max(h) as f64).min(1.0);
    if k >= 1.0 {
        return (bgra.to_vec(), w, h);
    }
    let (nw, nh) = (((w as f64 * k) as u32).max(1), ((h as f64 * k) as u32).max(1));
    let img = image::RgbaImage::from_raw(w, h, bgra.to_vec()).expect("tamaño de captura");
    let small = image::imageops::resize(&img, nw, nh, image::imageops::FilterType::Triangle);
    (small.into_raw(), nw, nh)
}

fn save_jpeg(path: &PathBuf, bgra: &[u8], w: u32, h: u32) -> Result<(), String> {
    let rgb: Vec<u8> = bgra.chunks_exact(4).flat_map(|p| [p[2], p[1], p[0]]).collect();
    let mut f = std::io::BufWriter::new(std::fs::File::create(path).map_err(|e| e.to_string())?);
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut f, 72)
        .encode(&rgb, w, h, image::ExtendedColorType::Rgb8)
        .map_err(|e| e.to_string())
}

/// Lo que se tocó según el OCR: la línea (o palabra) bajo el clic.
fn ocr_target(lines: &[sys::Line], x: f32, y: f32) -> String {
    let mut best: Option<(f32, &sys::Line, &str)> = None;
    for l in lines {
        for (word, r) in &l.words {
            let dx = (r.0 - x).max(0.0).max(x - (r.0 + r.2));
            let dy = (r.1 - y).max(0.0).max(y - (r.1 + r.3));
            let d = dx.hypot(dy);
            if d < 30.0 && best.map(|b| d < b.0).unwrap_or(true) {
                best = Some((d, l, word));
            }
        }
    }
    match best {
        Some((_, l, _)) if l.text.chars().count() <= 60 => l.text.clone(),
        Some((_, _, w)) => w.to_string(),
        None => String::new(),
    }
}

fn start(app: AppHandle) {
    let (tx, rx) = mpsc::channel::<Click>();
    sys::hook(tx);
    let _ = thread::Builder::new().name("bitacora".into()).spawn(move || worker(app, rx));
}

fn worker(app: AppHandle, rx: mpsc::Receiver<Click>) {
    sys::com_init();
    let uia = sys::Uia::new();
    let ocr = sys::Ocr::new();
    let mut last: Option<Last> = None;
    for c in rx {
        let b = app.state::<Bitacora>();
        if !b.on.load(Ordering::SeqCst) {
            continue;
        }
        let Some(w) = sys::window_at(c.x, c.y) else { continue };
        let f = b.filter.lock().map(|f| f.clone()).unwrap_or_default();
        let exe = w.exe.to_lowercase();
        if w.own || NEVER.contains(&exe.as_str()) || !f.apps.contains(&exe) {
            continue;
        }
        if !f.title.is_empty() && !w.title.to_lowercase().contains(&f.title) {
            continue;
        }
        let (name, password) = uia.as_ref().and_then(|u| u.at(c.x, c.y)).unwrap_or_default();
        if password {
            continue;
        }
        let state = app.state::<AppState>();
        let id = state.new_id();
        let fresh = last.as_ref().map(|l| l.hwnd != w.hwnd || l.at.elapsed() >= MIN_GAP).unwrap_or(true);

        let mut image = String::new();
        let mut text = String::new();
        let mut target = name.trim().chars().take(80).collect::<String>();
        let mut rect = None;
        if fresh {
            thread::sleep(SETTLE);
            if let Some(shot) = sys::capture(w.hwnd) {
                let sig = signature(&shot);
                let reuse = last.as_ref().filter(|l| l.hwnd == w.hwnd && same(&l.sig, &sig)).map(|l| l.image.clone());
                rect = Some((shot.left, shot.top, shot.w as i32, shot.h as i32));
                image = match reuse {
                    Some(img) => img,
                    None => {
                        let file = format!("{id}.jpg");
                        let (small, sw, sh) = shrink(&shot.bgra, shot.w, shot.h, MAX_W, u32::MAX);
                        let d = dir(&state);
                        let _ = std::fs::create_dir_all(&d);
                        if save_jpeg(&d.join(&file), &small, sw, sh).is_err() {
                            continue;
                        }
                        if let Some(o) = &ocr {
                            let (img, ow, oh) = shrink(&shot.bgra, shot.w, shot.h, u32::MAX, o.max_side());
                            let lines = o.read(&img, ow, oh);
                            text = lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join("\n");
                            if target.is_empty() {
                                let k = ow as f32 / shot.w as f32;
                                target = ocr_target(&lines, (c.x - shot.left) as f32 * k, (c.y - shot.top) as f32 * k);
                            }
                        }
                        file
                    }
                };
                last = Some(Last { hwnd: w.hwnd, at: Instant::now(), image: image.clone(), sig, rect: rect.unwrap() });
            }
        } else if let Some(l) = &last {
            image = l.image.clone();
            rect = Some(l.rect);
        }
        let (x, y) = match rect {
            Some((l, t, rw, rh)) if rw > 0 && rh > 0 => (
                Some(((c.x - l) as f64 / rw as f64).clamp(0.0, 1.0)),
                Some(((c.y - t) as f64 / rh as f64).clamp(0.0, 1.0)),
            ),
            _ => (None, None),
        };
        let e = Entry { id, ts: c.at, app: w.exe.clone(), title: w.title.clone(), target, image, x, y, ocr: text };
        if let Ok(db) = state.db.lock() {
            let _ = db.execute(
                "INSERT INTO bitacora (id,ts,app,title,target,image,x,y,ocr) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![e.id, e.ts, e.app, e.title, e.target, e.image, e.x, e.y, e.ocr],
            );
        }
        let _ = app.emit("bitacora-added", e.with_path(&dir(&state)));
    }
}

impl Entry {
    /// Para la interfaz: ruta completa de la captura (o vacía).
    fn with_path(mut self, d: &std::path::Path) -> Self {
        if !self.image.is_empty() {
            self.image = d.join(&self.image).to_string_lossy().into_owned();
        }
        self
    }
}

/* ===================== Comandos ===================== */

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Setup {
    on: bool,
    apps: Vec<String>,
    title: String,
    days: i64,
    running: Vec<sys::AppInfo>,
    ocr: bool,
}

#[tauri::command]
pub fn bitacora_setup(app: AppHandle) -> Setup {
    let state = app.state::<AppState>();
    let p = prefs(&state);
    Setup {
        on: app.state::<Bitacora>().on.load(Ordering::SeqCst),
        apps: p.apps,
        title: p.title,
        days: p.days,
        running: sys::running_apps(),
        ocr: sys::Ocr::available(),
    }
}

#[tauri::command]
pub fn set_bitacora(app: AppHandle, on: bool, apps: Vec<String>, title: String, days: i64) -> Result<(), String> {
    if ![7, 15, 30].contains(&days) {
        return Err("Opción no válida".into());
    }
    let apps: Vec<String> = apps.iter().map(|a| a.trim().to_lowercase()).filter(|a| !a.is_empty() && !a.contains(',')).collect();
    let on = on && !apps.is_empty();
    {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        for (k, v) in [
            ("bit_on", if on { "1" } else { "0" }.to_string()),
            ("bit_apps", apps.join(",")),
            ("bit_title", title.trim().to_string()),
            ("bit_days", days.to_string()),
        ] {
            db::set_setting(&db, k, &v).map_err(|e| e.to_string())?;
        }
    }
    if let Ok(mut f) = app.state::<Bitacora>().filter.lock() {
        *f = Filter { apps, title: title.trim().to_lowercase() };
    }
    purge(&app.state::<AppState>(), days);
    set_on(&app, on);
    Ok(())
}

#[tauri::command]
pub fn list_bitacora(state: State<AppState>, from: i64, to: i64) -> Result<Vec<Entry>, String> {
    let d = dir(&state);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut st = db
        .prepare("SELECT id,ts,app,title,target,image,x,y,ocr FROM bitacora WHERE ts>=?1 AND ts<?2 ORDER BY ts")
        .map_err(|e| e.to_string())?;
    let rows = st
        .query_map(params![from, to], |r| {
            Ok(Entry {
                id: r.get(0)?,
                ts: r.get(1)?,
                app: r.get(2)?,
                title: r.get(3)?,
                target: r.get(4)?,
                image: r.get(5)?,
                x: r.get(6)?,
                y: r.get(7)?,
                ocr: r.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(rows.flatten().map(|e| e.with_path(&d)).collect())
}

/// Borra lo registrado entre `from` y `to` (p. ej. un día completo).
#[tauri::command]
pub fn delete_bitacora(state: State<AppState>, from: i64, to: i64) -> Result<usize, String> {
    let (images, n) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let images: Vec<String> = db
            .prepare("SELECT DISTINCT image FROM bitacora WHERE ts>=?1 AND ts<?2 AND image != ''")
            .and_then(|mut st| st.query_map(params![from, to], |r| r.get::<_, String>(0)).map(|it| it.flatten().collect()))
            .map_err(|e| e.to_string())?;
        let n = db.execute("DELETE FROM bitacora WHERE ts>=?1 AND ts<?2", params![from, to]).map_err(|e| e.to_string())?;
        (images, n)
    };
    remove_unused(&state, images);
    Ok(n)
}

/* ===================== Windows ===================== */

#[cfg(windows)]
mod sys {
    use std::sync::{mpsc::Sender, Mutex, OnceLock};

    use serde::Serialize;
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HWND, LPARAM, POINT, RECT, WPARAM, LRESULT},
        Graphics::{
            Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
            Gdi::{
                BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SelectObject,
                BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS, SRCCOPY,
            },
        },
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION},
        },
        UI::WindowsAndMessaging::{
            CallNextHookEx, DispatchMessageW, EnumWindows, GetAncestor, GetMessageW, GetWindowLongW, GetWindowRect, GetWindowTextLengthW,
            GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, SetWindowsHookExW, TranslateMessage, WindowFromPoint, GA_ROOT,
            GWL_EXSTYLE, MSG, MSLLHOOKSTRUCT, WH_MOUSE_LL, WM_LBUTTONDOWN, WM_RBUTTONDOWN, WS_EX_TOOLWINDOW,
        },
    };

    use super::Click;

    static TX: OnceLock<Mutex<Sender<Click>>> = OnceLock::new();

    /// Gancho de mouse de bajo nivel: solo anota el punto y la hora; todo lo demás va en otro hilo
    /// para no frenar el mouse.
    unsafe extern "system" fn on_mouse(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        let msg = wp as u32;
        if code >= 0 && (msg == WM_LBUTTONDOWN || msg == WM_RBUTTONDOWN) {
            let m = &*(lp as *const MSLLHOOKSTRUCT);
            if let Some(Ok(tx)) = TX.get().map(|t| t.try_lock()) {
                let _ = tx.send(Click { x: m.pt.x, y: m.pt.y, at: crate::now_ms() });
            }
        }
        CallNextHookEx(std::ptr::null_mut(), code, wp, lp)
    }

    pub fn hook(tx: Sender<Click>) {
        if TX.set(Mutex::new(tx)).is_err() {
            return;
        }
        let _ = std::thread::Builder::new().name("bitacora-mouse".into()).spawn(|| unsafe {
            let h = SetWindowsHookExW(WH_MOUSE_LL, Some(on_mouse), GetModuleHandleW(std::ptr::null()), 0);
            if h.is_null() {
                return;
            }
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        });
    }

    fn text_of(hwnd: HWND) -> String {
        unsafe {
            let n = GetWindowTextLengthW(hwnd);
            if n <= 0 {
                return String::new();
            }
            let mut buf = vec![0u16; n as usize + 1];
            let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            String::from_utf16_lossy(&buf[..len.max(0) as usize])
        }
    }

    /// (pid, nombre del programa sin .exe)
    fn exe_of(hwnd: HWND) -> (u32, String) {
        unsafe {
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == 0 {
                return (0, String::new());
            }
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if h.is_null() {
                return (pid, String::new());
            }
            let mut buf = [0u16; 520];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len);
            CloseHandle(h);
            if ok == 0 {
                return (pid, String::new());
            }
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            (pid, std::path::Path::new(&path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default())
        }
    }

    pub struct Win {
        pub hwnd: isize,
        pub exe: String,
        pub title: String,
        pub own: bool,
    }

    /// La ventana principal que está bajo el punto del clic.
    pub fn window_at(x: i32, y: i32) -> Option<Win> {
        unsafe {
            let h = WindowFromPoint(POINT { x, y });
            if h.is_null() {
                return None;
            }
            let root = GetAncestor(h, GA_ROOT);
            let root = if root.is_null() { h } else { root };
            let (pid, exe) = exe_of(root);
            if exe.is_empty() {
                return None;
            }
            Some(Win { hwnd: root as isize, exe, title: text_of(root), own: pid == GetCurrentProcessId() })
        }
    }

    pub struct Shot {
        pub left: i32,
        pub top: i32,
        pub w: u32,
        pub h: u32,
        pub bgra: Vec<u8>,
    }

    /// Captura lo que se ve de la ventana en pantalla (también funciona con Chrome y apps con GPU).
    pub fn capture(hwnd: isize) -> Option<Shot> {
        unsafe {
            let hwnd = hwnd as HWND;
            let mut r: RECT = std::mem::zeroed();
            let hr = DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS as _, &mut r as *mut RECT as *mut _, std::mem::size_of::<RECT>() as u32);
            if hr != 0 && GetWindowRect(hwnd, &mut r) == 0 {
                return None;
            }
            let (w, h) = (r.right - r.left, r.bottom - r.top);
            if w < 8 || h < 8 {
                return None;
            }
            let sdc = GetDC(std::ptr::null_mut());
            let mdc = CreateCompatibleDC(sdc);
            let bmp = CreateCompatibleBitmap(sdc, w, h);
            let old = SelectObject(mdc, bmp);
            let ok = BitBlt(mdc, 0, 0, w, h, sdc, r.left, r.top, SRCCOPY | CAPTUREBLT);
            let mut bi: BITMAPINFO = std::mem::zeroed();
            bi.bmiHeader = BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB as _,
                ..std::mem::zeroed()
            };
            let mut buf = vec![0u8; (w * h * 4) as usize];
            let lines = GetDIBits(mdc, bmp, 0, h as u32, buf.as_mut_ptr() as *mut _, &mut bi, DIB_RGB_COLORS);
            SelectObject(mdc, old);
            DeleteObject(bmp);
            DeleteDC(mdc);
            ReleaseDC(std::ptr::null_mut(), sdc);
            if ok == 0 || lines == 0 {
                return None;
            }
            for p in buf.chunks_exact_mut(4) {
                p[3] = 255;
            }
            Some(Shot { left: r.left, top: r.top, w: w as u32, h: h as u32, bgra: buf })
        }
    }

    #[derive(Serialize)]
    pub struct AppInfo {
        exe: String,
        title: String,
    }

    unsafe extern "system" fn collect(hwnd: HWND, lp: LPARAM) -> i32 {
        let out = &mut *(lp as *mut Vec<AppInfo>);
        if IsWindowVisible(hwnd) == 0 || GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW != 0 {
            return 1;
        }
        let title = text_of(hwnd);
        if title.is_empty() {
            return 1;
        }
        let (pid, exe) = exe_of(hwnd);
        let low = exe.to_lowercase();
        if exe.is_empty() || pid == GetCurrentProcessId() || super::NEVER.contains(&low.as_str()) || out.iter().any(|a| a.exe.to_lowercase() == low) {
            return 1;
        }
        out.push(AppInfo { exe, title });
        1
    }

    /// Programas con ventanas abiertas ahora, para elegir cuáles registrar.
    pub fn running_apps() -> Vec<AppInfo> {
        let mut out: Vec<AppInfo> = Vec::new();
        unsafe { EnumWindows(Some(collect), &mut out as *mut _ as LPARAM) };
        out
    }

    /* ---- UI Automation: nombre del botón o campo tocado, y si es una contraseña ---- */

    pub fn com_init() {
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
    }

    pub struct Uia(windows::Win32::UI::Accessibility::IUIAutomation);

    impl Uia {
        pub fn new() -> Option<Self> {
            use windows::Win32::{
                System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
                UI::Accessibility::CUIAutomation,
            };
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok().map(Uia) }
        }

        pub fn at(&self, x: i32, y: i32) -> Option<(String, bool)> {
            unsafe {
                let el = self.0.ElementFromPoint(windows::Win32::Foundation::POINT { x, y }).ok()?;
                let password = el.CurrentIsPassword().map(|b| b.as_bool()).unwrap_or(false);
                let name = el.CurrentName().map(|s| s.to_string()).unwrap_or_default();
                Some((name, password))
            }
        }
    }

    /* ---- OCR de Windows (local) ---- */

    pub struct Line {
        pub text: String,
        /// (palabra, (x, y, ancho, alto)) en píxeles de la imagen leída
        pub words: Vec<(String, (f32, f32, f32, f32))>,
    }

    pub struct Ocr(windows::Media::Ocr::OcrEngine);

    impl Ocr {
        pub fn new() -> Option<Self> {
            windows::Media::Ocr::OcrEngine::TryCreateFromUserProfileLanguages().ok().map(Ocr)
        }

        pub fn available() -> bool {
            Self::new().is_some()
        }

        pub fn max_side(&self) -> u32 {
            windows::Media::Ocr::OcrEngine::MaxImageDimension().unwrap_or(2600)
        }

        pub fn read(&self, bgra: &[u8], w: u32, h: u32) -> Vec<Line> {
            self.try_read(bgra, w, h).unwrap_or_default()
        }

        fn try_read(&self, bgra: &[u8], w: u32, h: u32) -> windows::core::Result<Vec<Line>> {
            use windows::{
                Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap},
                Storage::Streams::DataWriter,
            };
            let writer = DataWriter::new()?;
            writer.WriteBytes(bgra)?;
            let buf = writer.DetachBuffer()?;
            let bmp = SoftwareBitmap::CreateCopyFromBuffer(&buf, BitmapPixelFormat::Bgra8, w as i32, h as i32)?;
            let res = self.0.RecognizeAsync(&bmp)?.join()?;
            let mut out = Vec::new();
            for line in res.Lines()? {
                let mut words = Vec::new();
                for word in line.Words()? {
                    let r = word.BoundingRect()?;
                    words.push((word.Text()?.to_string(), (r.X, r.Y, r.Width, r.Height)));
                }
                out.push(Line { text: line.Text()?.to_string(), words });
            }
            Ok(out)
        }
    }
}

#[cfg(not(windows))]
mod sys {
    use std::sync::mpsc::Sender;

    use serde::Serialize;

    use super::Click;

    pub fn hook(_: Sender<Click>) {}
    pub struct Win { pub hwnd: isize, pub exe: String, pub title: String, pub own: bool }
    pub fn window_at(_: i32, _: i32) -> Option<Win> { None }
    pub struct Shot { pub left: i32, pub top: i32, pub w: u32, pub h: u32, pub bgra: Vec<u8> }
    pub fn capture(_: isize) -> Option<Shot> { None }
    #[derive(Serialize)]
    pub struct AppInfo {}
    pub fn running_apps() -> Vec<AppInfo> { vec![] }
    pub fn com_init() {}
    pub struct Uia;
    impl Uia {
        pub fn new() -> Option<Self> { None }
        pub fn at(&self, _: i32, _: i32) -> Option<(String, bool)> { None }
    }
    pub struct Line { pub text: String, pub words: Vec<(String, (f32, f32, f32, f32))> }
    pub struct Ocr;
    impl Ocr {
        pub fn new() -> Option<Self> { None }
        pub fn available() -> bool { false }
        pub fn max_side(&self) -> u32 { 0 }
        pub fn read(&self, _: &[u8], _: u32, _: u32) -> Vec<Line> { vec![] }
    }
}
