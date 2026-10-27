// Grabación de reuniones y capacitaciones: micrófono + audio del PC (WASAPI loopback).
//
// El audio queda en %LOCALAPPDATA%\com.dani.lienzo\sesiones\<id>\audio.wav, estéreo a 16 kHz:
// canal izquierdo = micrófono (tú), derecho = audio del PC (los demás). Con los canales
// separados, la transcripción podrá distinguir quién habla sin reconocer voces.
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

use crate::{now_ms, AppState, Patch, RecItem};

/// Frecuencia que usa Whisper: grabar a 16 kHz evita convertir después.
pub const RATE: u32 = 16_000;
const SCOLS: &str = "id,kind,title,ts,ended_at,duration_ms,status,deleted_at,x,y,tone,transcript,error,acta";
const KINDS: [&str; 2] = ["reunion", "capacitacion"];

pub fn shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyR)
}

/* ===================== Sesiones guardadas ===================== */

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    id: String,
    kind: String,
    title: String,
    ts: i64,
    ended_at: Option<i64>,
    duration_ms: i64,
    status: String,
    deleted_at: Option<i64>,
    x: Option<f64>,
    y: Option<f64>,
    tone: String,
    transcript: String,
    error: String,
    acta: String,
    audio_path: String,
}

pub(crate) fn session_dir(state: &AppState, id: &str) -> PathBuf {
    state.sess_dir.join(id)
}

fn row_to_session(state: &AppState, r: &rusqlite::Row) -> rusqlite::Result<Session> {
    let id: String = r.get(0)?;
    Ok(Session {
        audio_path: session_dir(state, &id).join("audio.wav").to_string_lossy().into_owned(),
        id,
        kind: r.get(1)?,
        title: r.get(2)?,
        ts: r.get(3)?,
        ended_at: r.get(4)?,
        duration_ms: r.get(5)?,
        status: r.get(6)?,
        deleted_at: r.get(7)?,
        x: r.get(8)?,
        y: r.get(9)?,
        tone: r.get(10)?,
        transcript: r.get(11)?,
        error: r.get(12)?,
        acta: r.get(13)?,
    })
}

/// Avisa a la interfaz que una sesión cambió (estado, transcripción…).
pub(crate) fn emit_session(app: &AppHandle, id: &str) {
    if let Some(s) = get_session(&app.state::<AppState>(), id) {
        let _ = app.emit("session-updated", s);
    }
}

/// Id de la sesión que se está grabando ahora, si hay una.
pub(crate) fn recording_id(app: &AppHandle) -> Option<String> {
    let st = current(app);
    st.active.then_some(st.id)
}

fn get_session(state: &AppState, id: &str) -> Option<Session> {
    let db = state.db.lock().ok()?;
    db.query_row(&format!("SELECT {SCOLS} FROM sessions WHERE id=?1"), params![id], |r| row_to_session(state, r))
        .optional()
        .ok()
        .flatten()
}

#[tauri::command]
pub fn list_sessions(state: State<AppState>) -> Result<Vec<Session>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut st = db.prepare(&format!("SELECT {SCOLS} FROM sessions ORDER BY ts")).map_err(|e| e.to_string())?;
    let rows = st.query_map([], |r| row_to_session(&state, r)).map_err(|e| e.to_string())?;
    Ok(rows.flatten().collect())
}

fn apply_patch(db: &Connection, id: &str, patch: Patch) -> rusqlite::Result<()> {
    for (k, v) in patch {
        let col = match k.as_str() {
            "content" => "title",
            "sessionKind" => "kind",
            "deletedAt" => "deleted_at",
            "x" => "x",
            "y" => "y",
            "tone" => "tone",
            _ => continue,
        };
        if col == "kind" && !v.as_str().map(|s| KINDS.contains(&s)).unwrap_or(false) {
            continue;
        }
        let sql = format!("UPDATE sessions SET {col}=?1 WHERE id=?2");
        match v {
            serde_json::Value::Null => db.execute(&sql, params![Option::<i64>::None, id]),
            serde_json::Value::Number(n) if col == "x" || col == "y" => db.execute(&sql, params![n.as_f64(), id]),
            serde_json::Value::Number(n) => db.execute(&sql, params![n.as_i64(), id]),
            serde_json::Value::String(s) => db.execute(&sql, params![s, id]),
            _ => continue,
        }?;
    }
    Ok(())
}

#[tauri::command]
pub fn update_session(state: State<AppState>, id: String, patch: Patch) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    apply_patch(&db, &id, patch).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_session(app: AppHandle, state: State<AppState>, id: String) -> Result<(), String> {
    if current(&app).id == id {
        return Err("Detén la grabación antes de eliminarla".into());
    }
    crate::transcribe::cancel_if_current(&app, &id);
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute("DELETE FROM sessions WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
    }
    let _ = std::fs::remove_dir_all(session_dir(&state, &id));
    Ok(())
}

#[tauri::command]
pub fn open_session_folder(state: State<AppState>, id: String) -> Result<(), String> {
    let dir = session_dir(&state, &id);
    if !dir.is_dir() {
        return Err("La carpeta ya no existe".into());
    }
    std::process::Command::new("explorer").arg(&dir).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

/// Borra las sesiones de la papelera (todas, o solo las eliminadas antes de `before`) con su audio.
pub fn purge_trash(state: &AppState, before: Option<i64>) -> usize {
    let limit = before.unwrap_or(i64::MAX);
    let ids: Vec<String> = {
        let db = match state.db.lock() { Ok(d) => d, Err(_) => return 0 };
        let mut st = match db.prepare("SELECT id FROM sessions WHERE deleted_at IS NOT NULL AND deleted_at < ?1") {
            Ok(s) => s,
            Err(_) => return 0,
        };
        let v = st.query_map(params![limit], |r| r.get::<_, String>(0)).map(|it| it.flatten().collect()).unwrap_or_default();
        let _ = db.execute("DELETE FROM sessions WHERE deleted_at IS NOT NULL AND deleted_at < ?1", params![limit]);
        v
    };
    for id in &ids {
        let _ = std::fs::remove_dir_all(session_dir(state, id));
    }
    ids.len()
}

/// Si Lienzo se cerró de golpe grabando, el audio hasta el último guardado (cada 5 s) se conserva.
pub fn recover(state: &AppState) {
    let ids: Vec<String> = {
        let db = match state.db.lock() { Ok(d) => d, Err(_) => return };
        let mut st = match db.prepare("SELECT id FROM sessions WHERE status='grabando'") {
            Ok(s) => s,
            Err(_) => return,
        };
        let v = st.query_map([], |r| r.get::<_, String>(0)).map(|it| it.flatten().collect()).unwrap_or_default();
        v
    };
    for id in ids {
        let wav = session_dir(state, &id).join("audio.wav");
        // 44 bytes de cabecera; 4 bytes por instante (2 canales × 16 bits)
        let ms = std::fs::metadata(&wav).map(|m| m.len().saturating_sub(44) / 4 * 1000 / RATE as u64).unwrap_or(0) as i64;
        let status = if ms > 0 { "grabada" } else { "error" };
        if let Ok(db) = state.db.lock() {
            let _ = db.execute(
                "UPDATE sessions SET status=?1, duration_ms=?2, ended_at=ts+?2 WHERE id=?3",
                params![status, ms, id],
            );
        }
    }
}

/* ===================== Grabación en curso ===================== */

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct RecState {
    active: bool,
    id: String,
    kind: String,
    title: String,
    paused: bool,
    muted: bool,
    elapsed_ms: u64,
    mic: bool,
    system: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Tick {
    elapsed_ms: u64,
    mic: f32,
    system: f32,
    paused: bool,
}

struct Active {
    id: String,
    kind: String,
    title: String,
    mic: bool,
    system: bool,
    stop: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    muted: Arc<AtomicBool>,
    elapsed: Arc<AtomicU64>,
    handle: thread::JoinHandle<Result<u64, String>>,
}

#[derive(Default)]
pub struct Recorder(Mutex<Option<Active>>);

fn current(app: &AppHandle) -> RecState {
    let rec = app.state::<Recorder>();
    let g = match rec.0.lock() { Ok(g) => g, Err(_) => return RecState::default() };
    let st = match g.as_ref() {
        Some(a) => RecState {
            active: true,
            id: a.id.clone(),
            kind: a.kind.clone(),
            title: a.title.clone(),
            paused: a.paused.load(Ordering::SeqCst),
            muted: a.muted.load(Ordering::SeqCst),
            elapsed_ms: a.elapsed.load(Ordering::SeqCst),
            mic: a.mic,
            system: a.system,
        },
        None => RecState::default(),
    };
    st
}

fn announce(app: &AppHandle) {
    let st = current(app);
    if let Some(i) = app.try_state::<RecItem>() {
        let _ = i.0.set_text(if st.active { "Detener grabación  (Alt+Shift+R)" } else { "Grabar reunión  (Alt+Shift+R)" });
    }
    if let Some(t) = app.tray_by_id("tray") {
        let tip = if st.active { "Lienzo — grabando…".to_string() } else { format!("Lienzo {} — portapapeles", app.package_info().version) };
        let _ = t.set_tooltip(Some(tip));
    }
    let _ = app.emit("rec-state", st);
}

#[tauri::command]
pub fn rec_status(app: AppHandle) -> RecState {
    current(&app)
}

#[tauri::command]
pub fn rec_pause(app: AppHandle, paused: bool) {
    if let Ok(g) = app.state::<Recorder>().0.lock() {
        if let Some(a) = g.as_ref() {
            a.paused.store(paused, Ordering::SeqCst);
        }
    }
    announce(&app);
}

/// Silencia tu micrófono sin detener la grabación: ese tramo queda en silencio y solo se oye el PC.
#[tauri::command]
pub fn rec_mute(app: AppHandle, muted: bool) {
    if let Ok(g) = app.state::<Recorder>().0.lock() {
        if let Some(a) = g.as_ref() {
            a.muted.store(muted, Ordering::SeqCst);
        }
    }
    announce(&app);
}

#[tauri::command]
pub async fn rec_start(app: AppHandle, kind: String, title: String, mic: bool, system: bool) -> Result<Session, String> {
    tauri::async_runtime::spawn_blocking(move || start(&app, kind, title, mic, system)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn rec_stop(app: AppHandle) -> Result<Option<Session>, String> {
    tauri::async_runtime::spawn_blocking(move || stop(&app)).await.map_err(|e| e.to_string())?
}

/// Atajo y bandeja: empieza una reunión con micrófono y audio del PC, o detiene la actual.
pub fn toggle(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || {
        if current(&app).active {
            let _ = stop(&app);
        } else if let Err(e) = start(&app, "reunion".into(), String::new(), true, true) {
            let _ = app.emit("rec-error", e);
        }
    });
}

pub fn start(app: &AppHandle, kind: String, title: String, mic: bool, system: bool) -> Result<Session, String> {
    if !mic && !system {
        return Err("Elige al menos una fuente de audio".into());
    }
    let kind = if KINDS.contains(&kind.as_str()) { kind } else { "reunion".to_string() };
    let title = title.trim().to_string();
    let state = app.state::<AppState>();
    let rec = app.state::<Recorder>();
    let mut slot = rec.0.lock().map_err(|e| e.to_string())?;
    if slot.is_some() {
        return Err("Ya hay una grabación en curso".into());
    }

    let id = state.new_id();
    let dir = session_dir(&state, &id);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let stop = Arc::new(AtomicBool::new(false));
    let paused = Arc::new(AtomicBool::new(false));
    let muted = Arc::new(AtomicBool::new(false));
    let elapsed = Arc::new(AtomicU64::new(0));
    let (ready_tx, ready_rx) = mpsc::channel();
    let handle = {
        let (app, wav, stop, paused, muted, elapsed) =
            (app.clone(), dir.join("audio.wav"), stop.clone(), paused.clone(), muted.clone(), elapsed.clone());
        thread::Builder::new()
            .name("grabacion".into())
            .spawn(move || record(app, wav, mic, system, stop, paused, muted, elapsed, ready_tx))
            .map_err(|e| e.to_string())?
    };
    // Espera a que los dispositivos abran para avisar de inmediato si falta el micrófono, etc.
    match ready_rx.recv_timeout(Duration::from_secs(8)) {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            let _ = handle.join();
            let _ = std::fs::remove_dir_all(&dir);
            return Err(e);
        }
        Err(_) => {
            stop.store(true, Ordering::SeqCst);
            let _ = std::fs::remove_dir_all(&dir);
            return Err("El dispositivo de audio no respondió".into());
        }
    }

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute(
            "INSERT INTO sessions (id,kind,title,ts,status,mic,system) VALUES (?1,?2,?3,?4,'grabando',?5,?6)",
            params![id, kind, title, now_ms(), mic, system],
        )
        .map_err(|e| e.to_string())?;
    }
    *slot = Some(Active { id: id.clone(), kind, title, mic, system, stop, paused, muted, elapsed, handle });
    drop(slot);

    announce(app);
    let s = get_session(&state, &id).ok_or("no se pudo leer la sesión")?;
    let _ = app.emit("session-updated", s.clone());
    Ok(s)
}

pub fn stop(app: &AppHandle) -> Result<Option<Session>, String> {
    let active = {
        let rec = app.state::<Recorder>();
        let mut g = rec.0.lock().map_err(|e| e.to_string())?;
        g.take()
    };
    let Some(a) = active else { return Ok(None) };
    a.stop.store(true, Ordering::SeqCst);
    let res = a.handle.join().unwrap_or_else(|_| Err("la grabación se interrumpió".into()));

    let state = app.state::<AppState>();
    let (status, ms) = match &res {
        Ok(frames) => ("grabada", (frames * 1000 / RATE as u64) as i64),
        Err(_) => ("error", a.elapsed.load(Ordering::SeqCst) as i64),
    };
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute(
            "UPDATE sessions SET status=?1, duration_ms=?2, ended_at=?3 WHERE id=?4",
            params![status, ms, now_ms(), a.id],
        )
        .map_err(|e| e.to_string())?;
    }
    announce(app);
    let s = get_session(&state, &a.id);
    if let Some(s) = &s {
        let _ = app.emit("session-updated", s.clone());
    }
    res?;
    crate::transcribe::after_recording(app, &a.id);
    Ok(s)
}

/* ===================== Captura de audio ===================== */

/// Muestras de una fuente ya convertidas a 16 kHz mono, más el pico para el medidor.
#[derive(Default)]
struct Channel {
    buf: Mutex<Vec<f32>>,
    peak: Mutex<f32>,
}

impl Channel {
    fn take(&self) -> Vec<f32> {
        self.buf.lock().map(|mut b| std::mem::take(&mut *b)).unwrap_or_default()
    }
    fn peak(&self) -> f32 {
        self.peak.lock().map(|mut p| std::mem::replace(&mut *p, 0.0)).unwrap_or(0.0)
    }
}

/// Pasa de la frecuencia del dispositivo (casi siempre 48 kHz) a 16 kHz promediando
/// las muestras que caen en cada muestra de salida: un filtro simple, suficiente para voz.
struct Resampler {
    ratio: f64,
    pos: f64,
    acc: Vec<f32>,
}

impl Resampler {
    fn new(in_rate: u32) -> Self {
        Self { ratio: in_rate as f64 / RATE as f64, pos: 0.0, acc: Vec::new() }
    }
    fn push(&mut self, input: &[f32], out: &mut Vec<f32>) {
        self.acc.extend_from_slice(input);
        loop {
            let end = self.pos + self.ratio;
            if end > self.acc.len() as f64 {
                break;
            }
            let a = self.pos.floor() as usize;
            let b = (end.ceil() as usize).min(self.acc.len()).max(a + 1);
            out.push(self.acc[a..b].iter().sum::<f32>() / (b - a) as f32);
            self.pos = end;
        }
        let used = self.pos.floor() as usize;
        self.acc.drain(..used);
        self.pos -= used as f64;
    }
}

/// Abre un dispositivo. Con `loopback`, `dev` es una salida y se graba lo que suena en el PC.
/// Con `muted` activo se sigue leyendo el dispositivo pero se guarda silencio, para no perder el ritmo.
fn open_stream(dev: &cpal::Device, loopback: bool, ch: Arc<Channel>, paused: Arc<AtomicBool>, muted: Arc<AtomicBool>) -> Result<cpal::Stream, String> {
    let cfg = if loopback { dev.default_output_config() } else { dev.default_input_config() }.map_err(|e| e.to_string())?;
    let channels = cfg.channels().max(1) as usize;
    let fmt = cfg.sample_format();
    let config: cpal::StreamConfig = cfg.into();
    let mut rs = Resampler::new(config.sample_rate.0);

    macro_rules! build {
        ($t:ty, $conv:expr) => {{
            let (ch, paused, muted) = (ch.clone(), paused.clone(), muted.clone());
            let (mut mono, mut out) = (Vec::<f32>::new(), Vec::<f32>::new());
            dev.build_input_stream(
                &config,
                move |data: &[$t], _: &cpal::InputCallbackInfo| {
                    if paused.load(Ordering::Relaxed) {
                        return;
                    }
                    mono.clear();
                    let mut peak = 0f32;
                    let silent = muted.load(Ordering::Relaxed);
                    for frame in data.chunks(channels) {
                        let s = if silent { 0.0 } else { frame.iter().map(|&x| $conv(x)).sum::<f32>() / channels as f32 };
                        peak = peak.max(s.abs());
                        mono.push(s);
                    }
                    out.clear();
                    rs.push(&mono, &mut out);
                    if let Ok(mut b) = ch.buf.lock() {
                        b.extend_from_slice(&out);
                    }
                    if let Ok(mut p) = ch.peak.lock() {
                        *p = p.max(peak);
                    }
                },
                move |e: cpal::StreamError| eprintln!("audio: {e}"),
                None,
            )
        }};
    }

    let stream = match fmt {
        cpal::SampleFormat::F32 => build!(f32, |x: f32| x),
        cpal::SampleFormat::I16 => build!(i16, |x: i16| x as f32 / 32_768.0),
        cpal::SampleFormat::I32 => build!(i32, |x: i32| x as f32 / 2_147_483_648.0),
        cpal::SampleFormat::U16 => build!(u16, |x: u16| (x as f32 - 32_768.0) / 32_768.0),
        f => return Err(format!("Formato de audio no soportado: {f:?}")),
    }
    .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    Ok(stream)
}

fn pcm(s: Option<&f32>) -> i16 {
    (s.copied().unwrap_or(0.0).clamp(-1.0, 1.0) * 32_767.0) as i16
}

/// Hilo de grabación: los streams de cpal no se pueden mover entre hilos, así que
/// se abren, se usan y se cierran aquí. Devuelve cuántos instantes (frames) se escribieron.
#[allow(clippy::too_many_arguments)]
fn record(
    app: AppHandle,
    wav: PathBuf,
    mic: bool,
    system: bool,
    stop: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    muted: Arc<AtomicBool>,
    elapsed: Arc<AtomicU64>,
    ready: mpsc::Sender<Result<(), String>>,
) -> Result<u64, String> {
    let fail = |e: String| {
        let _ = ready.send(Err(e.clone()));
        Err(e)
    };
    let host = cpal::default_host();
    let (mic_ch, sys_ch) = (Arc::new(Channel::default()), Arc::new(Channel::default()));
    let mut streams = Vec::new();
    if mic {
        let Some(d) = host.default_input_device() else { return fail("No encontré un micrófono".into()) };
        match open_stream(&d, false, mic_ch.clone(), paused.clone(), muted) {
            Ok(s) => streams.push(s),
            Err(e) => return fail(format!("No se pudo abrir el micrófono: {e}. Revisa en Configuración → Privacidad → Micrófono que las aplicaciones de escritorio tengan acceso.")),
        }
    }
    if system {
        let Some(d) = host.default_output_device() else { return fail("No encontré la salida de audio del PC".into()) };
        match open_stream(&d, true, sys_ch.clone(), paused.clone(), Arc::new(AtomicBool::new(false))) {
            Ok(s) => streams.push(s),
            Err(e) => return fail(format!("No se pudo capturar el audio del PC: {e}")),
        }
    }
    let spec = hound::WavSpec { channels: 2, sample_rate: RATE, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
    let mut w = match hound::WavWriter::create(&wav, spec) {
        Ok(w) => w,
        Err(e) => return fail(e.to_string()),
    };
    let _ = ready.send(Ok(()));

    // El reloj del PC manda: el audio del PC no entrega nada mientras hay silencio,
    // así que el canal que se quede atrás se rellena con silencio para no desincronizarse.
    let margin = (RATE / 5) as u64; // 200 ms de espera para paquetes atrasados
    let (mut pm, mut ps) = (Vec::<f32>::new(), Vec::<f32>::new());
    let (mut written, mut active) = (0u64, Duration::ZERO);
    let (mut last, mut last_flush) = (Instant::now(), Instant::now());
    loop {
        thread::sleep(Duration::from_millis(100));
        let now = Instant::now();
        let is_paused = paused.load(Ordering::SeqCst);
        if !is_paused {
            active += now - last;
        }
        last = now;
        let done = stop.load(Ordering::SeqCst);

        pm.extend(mic_ch.take());
        ps.extend(sys_ch.take());
        let target = (active.as_secs_f64() * RATE as f64) as u64;
        let target = if done { target } else { target.saturating_sub(margin) };
        let n = target.saturating_sub(written) as usize;
        for i in 0..n {
            w.write_sample(pcm(pm.get(i))).map_err(|e| e.to_string())?;
            w.write_sample(pcm(ps.get(i))).map_err(|e| e.to_string())?;
        }
        pm.drain(..n.min(pm.len()));
        ps.drain(..n.min(ps.len()));
        // Si el reloj de un dispositivo va más rápido que el del PC, se descarta lo que sobre de 1 s
        for p in [&mut pm, &mut ps] {
            if p.len() > RATE as usize {
                let extra = p.len() - RATE as usize;
                p.drain(..extra);
            }
        }
        written += n as u64;

        let ms = active.as_millis() as u64;
        elapsed.store(ms, Ordering::SeqCst);
        let _ = app.emit("rec-tick", Tick { elapsed_ms: ms, mic: mic_ch.peak(), system: sys_ch.peak(), paused: is_paused });
        // Actualiza la cabecera del WAV: si Lienzo se cierra de golpe, el audio sigue siendo válido
        if last_flush.elapsed() > Duration::from_secs(5) {
            w.flush().map_err(|e| e.to_string())?;
            last_flush = Instant::now();
        }
        if done {
            break;
        }
    }
    drop(streams);
    w.finalize().map_err(|e| e.to_string())?;
    Ok(written)
}

