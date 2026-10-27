// Transcripción local con whisper.cpp: el audio nunca sale del equipo.
//
// Se transcribe una sola vez la mezcla de los dos canales y cada frase se atribuye a
// "Tú" (micrófono, canal izquierdo) o "Participantes" (audio del PC, canal derecho)
// según cuál de los dos tiene más energía en ese tramo.
use std::{
    io::Write,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager, State};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::{db, rec, AppState};

/// Revisión fija del repositorio de modelos: la huella SHA-256 de cada archivo no cambia.
const REV: &str = "5359861c739e955e79d9a303bcbc70fb988958b1";
/// Por debajo de esta energía (RMS) un tramo se considera silencio: Whisper a veces
/// "escucha" frases inventadas ahí y se descartan.
const SILENCE: f32 = 0.004;

pub struct Model {
    id: &'static str,
    file: &'static str,
    label: &'static str,
    hint: &'static str,
    size_mb: u32,
    sha256: &'static str,
}

static MODELS: [Model; 3] = [
    Model {
        id: "rapido",
        file: "ggml-base-q5_1.bin",
        label: "Rápido",
        hint: "Transcribe muy rápido, con más errores",
        size_mb: 57,
        sha256: "422f1ae452ade6f30a004d7e5c6a43195e4433bc370bf23fac9cc591f01a8898",
    },
    Model {
        id: "equilibrado",
        file: "ggml-small-q5_1.bin",
        label: "Equilibrado",
        hint: "Buen resultado en la mayoría de equipos (recomendado)",
        size_mb: 181,
        sha256: "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb",
    },
    Model {
        id: "preciso",
        file: "ggml-large-v3-turbo-q5_0.bin",
        label: "Preciso",
        hint: "El mejor resultado; tarda más en equipos sin GPU",
        size_mb: 547,
        sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
    },
];

fn find(id: &str) -> Option<&'static Model> {
    MODELS.iter().find(|m| m.id == id)
}

fn models_dir(state: &AppState) -> PathBuf {
    state.sess_dir.with_file_name("modelos")
}

fn model_path(state: &AppState, m: &Model) -> PathBuf {
    models_dir(state).join(m.file)
}

/* ===================== Preferencias ===================== */

struct Prefs {
    model: String,
    language: String,
    auto: bool,
}

fn prefs(state: &AppState) -> Prefs {
    let db = state.db.lock();
    let get = |k: &str, d: &str| db.as_ref().ok().and_then(|c| db::setting(c, k)).unwrap_or_else(|| d.to_string());
    Prefs { model: get("modelo", "equilibrado"), language: get("idioma", "es"), auto: get("auto", "1") == "1" }
}

/* ===================== Cola de trabajo ===================== */

#[derive(Default)]
pub struct Transcriber {
    wake: Mutex<bool>,
    cv: Condvar,
    current: Mutex<Option<(String, Arc<AtomicBool>)>>,
    download: Mutex<Option<Arc<AtomicBool>>>,
}

fn wake(app: &AppHandle) {
    let tr = app.state::<Transcriber>();
    if let Ok(mut w) = tr.wake.lock() {
        *w = true;
    }
    tr.cv.notify_one();
}

fn set_status(app: &AppHandle, id: &str, status: &str, error: &str) {
    if let Ok(db) = app.state::<AppState>().db.lock() {
        let _ = db.execute(
            "UPDATE sessions SET status=?1, error=?2, progress=0 WHERE id=?3",
            params![status, error, id],
        );
    }
    rec::emit_session(app, id);
}

/// Al abrir Lienzo: lo que quedó a medias se vuelve a poner en cola.
pub fn recover(state: &AppState) {
    if let Ok(db) = state.db.lock() {
        let _ = db.execute("UPDATE sessions SET status='en_cola', progress=0 WHERE status='transcribiendo'", []);
    }
}

/// Después de grabar: si está activada la transcripción automática y hay modelo, a la cola.
pub fn after_recording(app: &AppHandle, id: &str) {
    let state = app.state::<AppState>();
    let p = prefs(&state);
    if p.auto && find(&p.model).map(|m| model_path(&state, m).exists()).unwrap_or(false) {
        set_status(app, id, "en_cola", "");
        wake(app);
    }
}

/// Si se está transcribiendo esta sesión, la cancela (p. ej. antes de eliminarla).
pub fn cancel_if_current(app: &AppHandle, id: &str) {
    if let Ok(cur) = app.state::<Transcriber>().current.lock() {
        if let Some((cid, flag)) = cur.as_ref() {
            if cid == id {
                flag.store(true, Ordering::SeqCst);
            }
        }
    }
}

fn next_job(app: &AppHandle) -> Option<(String, &'static Model)> {
    let state = app.state::<AppState>();
    let m = find(&prefs(&state).model)?;
    if !model_path(&state, m).exists() {
        return None;
    }
    let db = state.db.lock().ok()?;
    let id: String = db
        .query_row(
            "SELECT id FROM sessions WHERE status='en_cola' AND deleted_at IS NULL ORDER BY ts LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()
        .ok()??;
    Some((id, m))
}

/// Un solo hilo transcribe, una sesión a la vez: Whisper ya usa varios núcleos.
pub fn start_worker(app: AppHandle) {
    let _ = thread::Builder::new().name("transcripcion".into()).spawn(move || loop {
        match next_job(&app) {
            Some((id, m)) => process(&app, &id, m),
            None => {
                let tr = app.state::<Transcriber>();
                let Ok(g) = tr.wake.lock() else { return };
                let waited = tr.cv.wait_timeout_while(g, Duration::from_secs(60), |w| !*w);
                if let Ok((mut g, _)) = waited {
                    *g = false;
                }
            }
        }
    });
}

#[derive(Serialize, Clone)]
struct Progress {
    id: String,
    pct: i32,
}

fn process(app: &AppHandle, id: &str, m: &'static Model) {
    let cancel = Arc::new(AtomicBool::new(false));
    if let Ok(mut cur) = app.state::<Transcriber>().current.lock() {
        *cur = Some((id.to_string(), cancel.clone()));
    }
    set_status(app, id, "transcribiendo", "");
    let res = transcribe(app, id, m, &cancel);
    if let Ok(mut cur) = app.state::<Transcriber>().current.lock() {
        *cur = None;
    }
    match res {
        Ok(text) => {
            if let Ok(db) = app.state::<AppState>().db.lock() {
                let _ = db.execute(
                    "UPDATE sessions SET status='transcrita', transcript=?1, error='', progress=1 WHERE id=?2",
                    params![text, id],
                );
            }
            rec::emit_session(app, id);
        }
        // Cancelada: vuelve a quedar como audio guardado, sin error
        Err(_) if cancel.load(Ordering::SeqCst) => set_status(app, id, "grabada", ""),
        Err(e) => set_status(app, id, "grabada", &e),
    }
}

/* ===================== Transcripción ===================== */

#[derive(Serialize, Deserialize, Clone)]
pub struct Segment {
    pub(crate) t0: i64,
    t1: i64,
    pub(crate) who: String,
    pub(crate) text: String,
}

#[derive(Serialize, Deserialize)]
pub struct Transcript {
    model: String,
    language: String,
    pub(crate) segments: Vec<Segment>,
}

fn rms(buf: &[f32], t0_ms: i64, t1_ms: i64) -> f32 {
    let per_ms = (rec::RATE / 1000) as i64;
    let a = (t0_ms.max(0) * per_ms) as usize;
    let b = ((t1_ms.max(0) * per_ms) as usize).min(buf.len());
    if b <= a {
        return 0.0;
    }
    (buf[a..b].iter().map(|x| x * x).sum::<f32>() / (b - a) as f32).sqrt()
}

fn clock(ms: i64) -> String {
    let s = ms / 1000;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}

fn transcribe(app: &AppHandle, id: &str, m: &Model, cancel: &Arc<AtomicBool>) -> Result<String, String> {
    let state = app.state::<AppState>();
    let dir = rec::session_dir(&state, id);
    let p = prefs(&state);
    let (title, kind): (String, String) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.query_row("SELECT title, kind FROM sessions WHERE id=?1", params![id], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?
    };

    // Audio estéreo de 16 kHz: izquierda = micrófono, derecha = PC
    let mut reader = hound::WavReader::open(dir.join("audio.wav")).map_err(|e| format!("No se pudo abrir el audio: {e}"))?;
    let spec = reader.spec();
    if spec.sample_rate != rec::RATE || spec.channels != 2 || spec.bits_per_sample != 16 {
        return Err("El audio no tiene el formato esperado".into());
    }
    let (mut left, mut right) = (Vec::new(), Vec::new());
    for (i, s) in reader.samples::<i16>().enumerate() {
        let v = s.map_err(|e| e.to_string())? as f32 / 32_768.0;
        if i % 2 == 0 { left.push(v) } else { right.push(v) }
    }
    if left.len() < rec::RATE as usize {
        return Err("La grabación es demasiado corta para transcribirla".into());
    }
    let mix: Vec<f32> = left.iter().zip(&right).map(|(a, b)| (a + b).clamp(-1.0, 1.0)).collect();

    let ctx = WhisperContext::new_with_params(model_path(&state, m), WhisperContextParameters::default())
        .map_err(|e| format!("No se pudo cargar el modelo: {e}"))?;
    let mut ws = ctx.create_state().map_err(|e| e.to_string())?;

    let mut fp = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    let language = if p.language == "auto" { "auto".to_string() } else { p.language.clone() };
    fp.set_language(Some(&language));
    let cores = thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    fp.set_n_threads(cores.saturating_sub(1).clamp(1, 8) as i32);
    fp.set_print_special(false);
    fp.set_print_progress(false);
    fp.set_print_realtime(false);
    fp.set_print_timestamps(false);
    // Sin contexto previo: evita que Whisper entre en bucles repitiendo frases en audios largos
    fp.set_no_context(true);
    fp.set_suppress_nst(true);
    let what = if kind == "capacitacion" { "Capacitación" } else { "Reunión de trabajo" };
    let prompt = if title.is_empty() { format!("{what}.") } else { format!("{what}: {title}.") };
    fp.set_initial_prompt(&prompt);

    let (app2, id2) = (app.clone(), id.to_string());
    let mut last = -1;
    fp.set_progress_callback_safe(move |pct: i32| {
        if pct != last {
            last = pct;
            let _ = app2.emit("transcribe-progress", Progress { id: id2.clone(), pct });
        }
    });
    let flag = cancel.clone();
    let abort: Box<dyn FnMut() -> bool> = Box::new(move || flag.load(Ordering::Relaxed));
    fp.set_abort_callback_safe(abort);

    ws.full(fp, &mix).map_err(|e| {
        if cancel.load(Ordering::SeqCst) { "Cancelada".to_string() } else { format!("La transcripción falló: {e}") }
    })?;

    let mut segments = Vec::new();
    for i in 0..ws.full_n_segments() {
        let Some(s) = ws.get_segment(i) else { continue };
        let text = s.to_str_lossy().map(|t| t.trim().to_string()).unwrap_or_default();
        if text.is_empty() {
            continue;
        }
        // Las marcas de Whisper vienen en centésimas de segundo
        let (t0, t1) = (s.start_timestamp() * 10, s.end_timestamp() * 10);
        let (el, er) = (rms(&left, t0, t1), rms(&right, t0, t1));
        if el.max(er) < SILENCE {
            continue;
        }
        let who = if el >= er { "yo" } else { "otros" };
        segments.push(Segment { t0, t1, who: who.into(), text });
    }
    if segments.is_empty() {
        return Err("No se detectó voz en la grabación".into());
    }

    // Si todo lo dijo una sola fuente, no se marca quién habla
    let both = segments.iter().any(|s| s.who == "yo") && segments.iter().any(|s| s.who == "otros");
    let mut txt = String::new();
    let mut prev = "";
    for s in &segments {
        let label = if !both { "" } else if s.who == "yo" { "Tú: " } else { "Participantes: " };
        if s.who != prev || !both {
            if !txt.is_empty() {
                txt.push('\n');
            }
            txt.push_str(&format!("[{}] {label}", clock(s.t0)));
        } else {
            txt.push(' ');
        }
        txt.push_str(&s.text);
        prev = s.who.as_str();
    }
    let data = Transcript { model: m.id.into(), language: p.language.clone(), segments };
    let json = serde_json::to_string(&data).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("transcripcion.json"), json).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("transcripcion.txt"), &txt).map_err(|e| e.to_string())?;
    Ok(txt)
}

/* ===================== Comandos ===================== */

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelView {
    id: &'static str,
    label: &'static str,
    hint: &'static str,
    size_mb: u32,
    installed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Setup {
    models: Vec<ModelView>,
    model: String,
    language: String,
    auto: bool,
    downloading: bool,
}

#[tauri::command]
pub fn transcribe_setup(state: State<AppState>, tr: State<Transcriber>) -> Setup {
    let p = prefs(&state);
    Setup {
        models: MODELS
            .iter()
            .map(|m| ModelView { id: m.id, label: m.label, hint: m.hint, size_mb: m.size_mb, installed: model_path(&state, m).exists() })
            .collect(),
        model: p.model,
        language: p.language,
        auto: p.auto,
        downloading: tr.download.lock().map(|d| d.is_some()).unwrap_or(false),
    }
}

#[tauri::command]
pub fn set_transcribe_prefs(app: AppHandle, model: String, language: String, auto: bool) -> Result<(), String> {
    if find(&model).is_none() || !["es", "en", "auto"].contains(&language.as_str()) {
        return Err("Opción no válida".into());
    }
    {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        for (k, v) in [("modelo", model.as_str()), ("idioma", language.as_str()), ("auto", if auto { "1" } else { "0" })] {
            db::set_setting(&db, k, v).map_err(|e| e.to_string())?;
        }
    }
    wake(&app);
    Ok(())
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DownloadProgress {
    id: String,
    done: u64,
    total: u64,
}

/// Descarga el modelo desde Hugging Face, verifica su SHA-256 y solo entonces lo deja disponible.
#[tauri::command]
pub async fn download_model(app: AppHandle, id: String) -> Result<(), String> {
    let m = find(&id).ok_or("Modelo desconocido")?;
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let tr = app.state::<Transcriber>();
        let mut d = tr.download.lock().map_err(|e| e.to_string())?;
        if d.is_some() {
            return Err("Ya hay una descarga en curso".into());
        }
        *d = Some(cancel.clone());
    }
    let res = fetch(&app, m, &cancel).await;
    if let Ok(mut d) = app.state::<Transcriber>().download.lock() {
        *d = None;
    }
    if res.is_ok() {
        wake(&app);
    }
    res
}

async fn fetch(app: &AppHandle, m: &Model, cancel: &AtomicBool) -> Result<(), String> {
    let (dir, dest) = {
        let state = app.state::<AppState>();
        (models_dir(&state), model_path(&state, m))
    };
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let part = dir.join(format!("{}.part", m.file));
    let url = format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/{REV}/{}", m.file);
    let mut resp = reqwest::get(&url)
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("No se pudo descargar el modelo: {e}"))?;
    let total = resp.content_length().unwrap_or(m.size_mb as u64 * 1_048_576);
    let mut file = std::fs::File::create(&part).map_err(|e| e.to_string())?;
    let mut sha = Sha256::new();
    let (mut done, mut last) = (0u64, Instant::now());
    let result: Result<(), String> = async {
        while let Some(chunk) = resp.chunk().await.map_err(|e| format!("La descarga se interrumpió: {e}"))? {
            if cancel.load(Ordering::SeqCst) {
                return Err("Descarga cancelada".into());
            }
            file.write_all(&chunk).map_err(|e| e.to_string())?;
            sha.update(&chunk);
            done += chunk.len() as u64;
            if last.elapsed() > Duration::from_millis(250) {
                last = Instant::now();
                let _ = app.emit("model-progress", DownloadProgress { id: m.id.into(), done, total });
            }
        }
        file.flush().map_err(|e| e.to_string())
    }
    .await;
    drop(file);
    if let Err(e) = result {
        let _ = std::fs::remove_file(&part);
        return Err(e);
    }
    let got: String = sha.finalize().iter().map(|b| format!("{b:02x}")).collect();
    if got != m.sha256 {
        let _ = std::fs::remove_file(&part);
        return Err("El modelo llegó dañado; vuelve a intentarlo".into());
    }
    std::fs::rename(&part, &dest).map_err(|e| e.to_string())?;
    let _ = app.emit("model-progress", DownloadProgress { id: m.id.into(), done: total, total });
    Ok(())
}

#[tauri::command]
pub fn cancel_download(tr: State<Transcriber>) {
    if let Ok(d) = tr.download.lock() {
        if let Some(flag) = d.as_ref() {
            flag.store(true, Ordering::SeqCst);
        }
    }
}

#[tauri::command]
pub fn delete_model(state: State<AppState>, tr: State<Transcriber>, id: String) -> Result<(), String> {
    let m = find(&id).ok_or("Modelo desconocido")?;
    if tr.current.lock().map(|c| c.is_some()).unwrap_or(false) && prefs(&state).model == id {
        return Err("Espera a que termine la transcripción en curso".into());
    }
    std::fs::remove_file(model_path(&state, m)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn transcribe_session(app: AppHandle, id: String) -> Result<(), String> {
    if rec::recording_id(&app).as_deref() == Some(id.as_str()) {
        return Err("Detén la grabación antes de transcribirla".into());
    }
    set_status(&app, &id, "en_cola", "");
    wake(&app);
    Ok(())
}

#[tauri::command]
pub fn cancel_transcription(app: AppHandle, id: String) {
    cancel_if_current(&app, &id);
    // Si aún estaba en cola, sale de ella
    let queued = match app.state::<AppState>().db.lock() {
        Ok(db) => db.execute("UPDATE sessions SET status='grabada' WHERE id=?1 AND status='en_cola'", params![id]).unwrap_or(0),
        Err(_) => 0,
    };
    if queued > 0 {
        rec::emit_session(&app, &id);
    }
}

#[tauri::command]
pub fn get_transcript(state: State<AppState>, id: String) -> Result<Option<Transcript>, String> {
    let path = rec::session_dir(&state, &id).join("transcripcion.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map(Some).map_err(|e| e.to_string())
}
