// A partir de la bitácora: informe de un día con Gemini y tutoriales en Word con las capturas.
//
// Al informe solo va texto (hora, app, ventana, lo que se tocó y parte del texto en pantalla),
// nunca las imágenes, y solo cuando lo pides.
use docx_rs::{Docx, Paragraph, Pic, Run};
use rusqlite::params;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager, State};

use crate::{acta, export, AppState};

/// Tope de texto que se envía para un día: deja fuera el texto de pantalla cuando se llega al límite.
const BUDGET: usize = 400_000;

struct Row {
    ts: i64,
    app: String,
    title: String,
    target: String,
    image: String,
    x: Option<f64>,
    y: Option<f64>,
    ocr: String,
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<Row> {
    Ok(Row {
        ts: r.get(0)?,
        app: r.get(1)?,
        title: r.get(2)?,
        target: r.get(3)?,
        image: r.get(4)?,
        x: r.get(5)?,
        y: r.get(6)?,
        ocr: r.get(7)?,
    })
}

const RCOLS: &str = "ts,app,title,target,image,x,y,ocr";

/// Hora local hh:mm:ss; `tz` son los minutos que da getTimezoneOffset() en la interfaz.
fn hms(ts: i64, tz: i64) -> String {
    let s = (ts - tz * 60_000).rem_euclid(86_400_000) / 1000;
    format!("{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

fn squash(s: &str, max: usize) -> String {
    let t = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.chars().count() > max { format!("{}…", t.chars().take(max).collect::<String>()) } else { t }
}

/* ===================== Informe del día ===================== */

fn report_schema() -> Value {
    let s = json!({ "type": "STRING" });
    json!({
        "type": "OBJECT",
        "properties": {
            "resumen": s,
            "bloques": { "type": "ARRAY", "items": { "type": "OBJECT",
                "properties": { "desde": s, "hasta": s, "app": s, "actividad": s },
                "required": ["desde", "hasta", "app", "actividad"] } },
            "atencion": { "type": "ARRAY", "items": { "type": "OBJECT",
                "properties": { "hora": s, "detalle": s },
                "required": ["hora", "detalle"] } },
        },
        "required": ["resumen", "bloques", "atencion"],
    })
}

#[tauri::command]
pub fn get_informe(state: State<AppState>, day: i64) -> Option<String> {
    let db = state.db.lock().ok()?;
    db.query_row("SELECT json FROM informes WHERE day=?1", params![day], |r| r.get(0)).ok()
}

/// `day` es el inicio del día en la hora local (ms); `label`, la fecha legible ("martes 27 de octubre").
#[tauri::command]
pub async fn generate_informe(app: AppHandle, day: i64, to: i64, tz: i64, label: String) -> Result<String, String> {
    let rows: Vec<Row> = {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut st = db
            .prepare(&format!("SELECT {RCOLS} FROM bitacora WHERE ts>=?1 AND ts<?2 ORDER BY ts"))
            .map_err(|e| e.to_string())?;
        let v = st.query_map(params![day, to], row).map_err(|e| e.to_string())?.flatten().collect();
        v
    };
    if rows.is_empty() {
        return Err("No hay nada registrado ese día".into());
    }
    let mut log = String::new();
    for r in &rows {
        log.push_str(&format!("{} · {} · {}", hms(r.ts, tz), r.app, squash(&r.title, 120)));
        if !r.target.is_empty() {
            log.push_str(&format!(" · clic en «{}»", squash(&r.target, 80)));
        }
        if !r.ocr.is_empty() && log.len() < BUDGET {
            log.push_str(&format!("\n    Pantalla: {}", squash(&r.ocr, 300)));
        }
        log.push('\n');
    }
    let prompt = format!(
        "Ayudas a una persona a recordar qué hizo en su computador para encontrar posibles errores. \
         Abajo está su bitácora del {label}: cada línea es un clic con la hora, el programa, el título de la \
         ventana y lo que tocó; algunas incluyen parte del texto que se veía en pantalla.\n\
         Escribe en español claro, en segunda persona (\"abriste\", \"editaste\"), sin inventar nada que no esté \
         en la bitácora.\n\n\
         Devuelve:\n\
         - resumen: 3 a 6 frases con lo principal del día.\n\
         - bloques: la jornada en tramos de actividad, en orden, con hora de inicio y fin (hh:mm), el programa y \
           qué hiciste en concreto (nombres de archivos, clientes, pantallas, si aparecen).\n\
         - atencion: acciones que conviene revisar porque pudieron causar un error: eliminaciones, envíos, \
           cambios de valores o estados, mensajes de error en pantalla, cierres sin guardar. Hora (hh:mm:ss) y \
           detalle. Vacío si no hay nada.\n\n\
         Bitácora:\n{log}"
    );
    let json = acta::gemini(prompt, report_schema()).await?;
    {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute(
            "INSERT INTO informes(day,json,ts) VALUES(?1,?2,?3) ON CONFLICT(day) DO UPDATE SET json=excluded.json, ts=excluded.ts",
            params![day, json, crate::now_ms()],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(json)
}

/* ===================== Tutorial ===================== */

/// Marca el clic con un aro rojo sobre la captura.
fn mark(img: &mut image::RgbImage, x: f64, y: f64) {
    let (w, h) = img.dimensions();
    let (cx, cy) = (x * w as f64, y * h as f64);
    let (r_out, r_in) = (16.0, 11.0);
    let x0 = (cx - r_out).max(0.0) as u32;
    let y0 = (cy - r_out).max(0.0) as u32;
    let x1 = ((cx + r_out) as u32).min(w.saturating_sub(1));
    let y1 = ((cy + r_out) as u32).min(h.saturating_sub(1));
    for py in y0..=y1 {
        for px in x0..=x1 {
            let d = ((px as f64 - cx).powi(2) + (py as f64 - cy).powi(2)).sqrt();
            if d <= r_out && d >= r_in {
                img.put_pixel(px, py, image::Rgb([229, 72, 77]));
            }
        }
    }
}

fn tutorial_schema() -> Value {
    let s = json!({ "type": "STRING" });
    json!({
        "type": "OBJECT",
        "properties": { "titulo": s, "pasos": { "type": "ARRAY", "items": s } },
        "required": ["titulo", "pasos"],
    })
}

/// Arma un tutorial en Word con los clics elegidos (uno por paso, con su captura).
/// Con `ai`, Gemini redacta cada paso como una instrucción; si no, se describe el clic tal cual.
#[tauri::command]
pub async fn bitacora_tutorial(app: AppHandle, ids: Vec<String>, title: String, ai: bool) -> Result<String, String> {
    let (rows, dir) = {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let mut st = db.prepare(&format!("SELECT {RCOLS} FROM bitacora WHERE id=?1")).map_err(|e| e.to_string())?;
        let mut rows: Vec<Row> = ids.iter().filter_map(|id| st.query_row(params![id], row).ok()).collect();
        rows.sort_by_key(|r| r.ts);
        (rows, state.sess_dir.with_file_name("bitacora"))
    };
    // Dos clics seguidos en lo mismo cuentan como un solo paso
    let mut steps: Vec<Row> = Vec::new();
    for r in rows {
        if steps.last().map(|p| p.image == r.image && p.target == r.target).unwrap_or(false) {
            continue;
        }
        steps.push(r);
    }
    if steps.is_empty() {
        return Err("No hay pasos para el tutorial".into());
    }

    let plain: Vec<String> = steps
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let what = if r.target.is_empty() { "Haz clic en el punto marcado".to_string() } else { format!("Haz clic en «{}»", r.target) };
            let changed = i == 0 || steps[i - 1].app != r.app || steps[i - 1].title != r.title;
            if changed { format!("{what} ({}).", r.title) } else { format!("{what}.") }
        })
        .collect();
    let (mut doc_title, mut texts) = (title.trim().to_string(), plain.clone());
    if ai {
        let list: String = steps
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let screen = if r.ocr.is_empty() { String::new() } else { format!(" | pantalla: {}", squash(&r.ocr, 250)) };
                format!("{}. {} · {} · clic en «{}»{screen}\n", i + 1, r.app, squash(&r.title, 100), r.target)
            })
            .collect();
        let prompt = format!(
            "Convierte estos {n} clics, en orden, en un tutorial paso a paso en español. Devuelve exactamente {n} \
             pasos, uno por clic y en el mismo orden: cada uno una instrucción breve en imperativo (\"Abre…\", \
             \"Haz clic en…\", \"Escribe…\") que explique qué hacer y para qué si se deduce del contexto. \
             No inventes datos. «titulo» es un nombre corto para el tutorial{hint}.\n\nClics:\n{list}",
            n = steps.len(),
            hint = if doc_title.is_empty() { String::new() } else { format!(" (la persona sugirió «{doc_title}»)") }
        );
        let v: Value = serde_json::from_str(&acta::gemini(prompt, tutorial_schema()).await?).map_err(|e| e.to_string())?;
        let pasos: Vec<String> = v["pasos"].as_array().map(|a| a.iter().filter_map(|p| p.as_str().map(str::to_string)).collect()).unwrap_or_default();
        if pasos.len() == steps.len() {
            texts = pasos;
        }
        if doc_title.is_empty() {
            doc_title = v["titulo"].as_str().unwrap_or("").trim().to_string();
        }
    }
    if doc_title.is_empty() {
        doc_title = "Tutorial".into();
    }

    let mut doc = Docx::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(&doc_title).bold().size(36)));
    for (i, (r, text)) in steps.iter().zip(&texts).enumerate() {
        doc = doc.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(format!("Paso {}.  ", i + 1)).bold().size(24))
                .add_run(Run::new().add_text(text).size(24)),
        );
        let Ok(raw) = std::fs::read(dir.join(&r.image)) else { continue };
        let Ok(img) = image::load_from_memory(&raw) else { continue };
        let mut img = img.to_rgb8();
        if let (Some(x), Some(y)) = (r.x, r.y) {
            mark(&mut img, x, y);
        }
        let (w, h) = img.dimensions();
        let mut jpg = Vec::new();
        if image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpg, 80)
            .encode(img.as_raw(), w, h, image::ExtendedColorType::Rgb8)
            .is_err()
        {
            continue;
        }
        // 6 pulgadas de ancho como máximo (914 400 EMU por pulgada)
        let ew = 5_486_400u32.min(w * 9_525);
        let eh = (ew as u64 * h as u64 / w.max(1) as u64) as u32;
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_image(Pic::new(&jpg).size(ew, eh))));
    }
    let path = export::out_path(&app, &format!("Tutorial - {doc_title}"), "Tutorial", "docx")?;
    let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    doc.build().pack(file).map_err(|e| format!("No se pudo crear el documento: {e}"))?;
    export::reveal(&path);
    Ok(path.to_string_lossy().into_owned())
}
