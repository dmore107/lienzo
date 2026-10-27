// Exportar una grabación (acta + transcripción) a Markdown o Word.
//
// Los archivos quedan en Documentos\Lienzo y se muestran en el Explorador al terminar.
use std::path::PathBuf;

use docx_rs::{Docx, Paragraph, Run};
use rusqlite::params;
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{rec, transcribe, AppState};

/// Lo que se exporta, independiente del formato.
enum Block {
    Title(String),
    Meta(String),
    Heading(String),
    Text(String),
    Bullet(String),
    Task(String, String),
    Numbered(usize, String),
    Line(String, String, String),
}

fn s(v: &Value) -> String {
    v.as_str().unwrap_or("").trim().to_string()
}

fn items(v: &Value) -> &[Value] {
    v.as_array().map(|a| a.as_slice()).unwrap_or(&[])
}

fn clock(ms: i64) -> String {
    let t = ms / 1000;
    if t >= 3600 {
        format!("{}:{:02}:{:02}", t / 3600, t / 60 % 60, t % 60)
    } else {
        format!("{:02}:{:02}", t / 60, t % 60)
    }
}

/// Arma el documento: encabezado, acta (si hay) y transcripción por párrafos.
fn blocks(kind: &str, title: &str, when: &str, duration_ms: i64, acta: &str, tr: Option<transcribe::Transcript>) -> Vec<Block> {
    let what = if kind == "capacitacion" { "Capacitación" } else { "Reunión" };
    let mut b = vec![
        Block::Title(if title.is_empty() { format!("{what} sin título") } else { title.to_string() }),
        Block::Meta(format!("{what} · {when} · {}", clock(duration_ms))),
    ];
    if let Ok(a) = serde_json::from_str::<Value>(acta) {
        b.push(Block::Heading("Resumen".into()));
        b.push(Block::Text(s(&a["resumen"])));
        if kind == "capacitacion" {
            b.push(Block::Heading("Conceptos".into()));
            for c in items(&a["conceptos"]) {
                b.push(Block::Bullet(format!("{}: {}", s(&c["concepto"]), s(&c["explicacion"]))));
            }
            b.push(Block::Heading("Pasos".into()));
            for (i, p) in items(&a["pasos"]).iter().enumerate() {
                b.push(Block::Numbered(i + 1, s(p)));
            }
        } else {
            b.push(Block::Heading("Decisiones".into()));
            for d in items(&a["decisiones"]) {
                b.push(Block::Bullet(s(d)));
            }
            b.push(Block::Heading("Tareas".into()));
            for t in items(&a["tareas"]) {
                let who: Vec<String> = [s(&t["responsable"]), s(&t["fecha"])].into_iter().filter(|x| !x.is_empty()).collect();
                b.push(Block::Task(s(&t["tarea"]), who.join(" · ")));
            }
        }
        b.push(Block::Heading("Temas".into()));
        for t in items(&a["temas"]) {
            b.push(Block::Line(s(&t["minuto"]), String::new(), s(&t["tema"])));
        }
    }
    if let Some(tr) = tr {
        b.push(Block::Heading("Transcripción".into()));
        let both = tr.segments.iter().any(|x| x.who == "yo") && tr.segments.iter().any(|x| x.who == "otros");
        // Párrafos como en el visor: cambia quien habla o pasan 45 s
        let mut cur: Option<(i64, String, String)> = None;
        let flush = |c: Option<(i64, String, String)>, b: &mut Vec<Block>| {
            if let Some((t0, who, text)) = c {
                let label = if !both { "" } else if who == "yo" { "Tú" } else { "Participantes" };
                b.push(Block::Line(clock(t0), label.into(), text));
            }
        };
        for seg in &tr.segments {
            match &mut cur {
                Some((t0, who, text)) if *who == seg.who && seg.t0 - *t0 < 45_000 => {
                    text.push(' ');
                    text.push_str(&seg.text);
                }
                _ => {
                    flush(cur.take(), &mut b);
                    cur = Some((seg.t0, seg.who.clone(), seg.text.clone()));
                }
            }
        }
        flush(cur, &mut b);
    }
    b
}

fn markdown(b: &[Block]) -> String {
    let mut out = String::new();
    for x in b {
        let line = match x {
            Block::Title(t) => format!("# {t}\n"),
            Block::Meta(t) => format!("_{t}_\n"),
            Block::Heading(t) => format!("\n## {t}\n"),
            Block::Text(t) => format!("{t}\n"),
            Block::Bullet(t) => format!("- {t}"),
            Block::Task(t, who) if who.is_empty() => format!("- [ ] {t}"),
            Block::Task(t, who) => format!("- [ ] {t} ({who})"),
            Block::Numbered(n, t) => format!("{n}. {t}"),
            Block::Line(at, who, t) if who.is_empty() => format!("**[{at}]** {t}\n"),
            Block::Line(at, who, t) => format!("**[{at}] {who}:** {t}\n"),
        };
        out.push_str(&line);
        out.push('\n');
    }
    out
}

fn docx(b: &[Block], path: &PathBuf) -> Result<(), String> {
    let run = |t: &str| Run::new().add_text(t).size(22);
    let gray = |t: &str| Run::new().add_text(t).size(20).color("808080");
    let mut doc = Docx::new();
    for x in b {
        let p = match x {
            Block::Title(t) => Paragraph::new().add_run(Run::new().add_text(t).bold().size(36)),
            Block::Meta(t) => Paragraph::new().add_run(gray(t)),
            Block::Heading(t) => Paragraph::new().add_run(Run::new().add_text(t).bold().size(28)),
            Block::Text(t) => Paragraph::new().add_run(run(t)),
            Block::Bullet(t) => Paragraph::new().add_run(run(&format!("•  {t}"))),
            Block::Task(t, who) => {
                let p = Paragraph::new().add_run(run(&format!("☐  {t}")));
                if who.is_empty() { p } else { p.add_run(gray(&format!("   {who}"))) }
            }
            Block::Numbered(n, t) => Paragraph::new().add_run(run(&format!("{n}.  {t}"))),
            Block::Line(at, who, t) => {
                let p = Paragraph::new().add_run(gray(&format!("{at}   ")));
                let p = if who.is_empty() { p } else { p.add_run(Run::new().add_text(format!("{who}: ")).bold().size(22)) };
                p.add_run(run(t))
            }
        };
        doc = doc.add_paragraph(p);
    }
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    doc.build().pack(file).map_err(|e| format!("No se pudo crear el documento: {e}"))?;
    Ok(())
}

/// Quita lo que Windows no admite en un nombre de archivo.
fn file_name(title: &str, fallback: &str) -> String {
    let clean: String = title
        .chars()
        .map(|c| if "\\/:*?\"<>|".contains(c) || c.is_control() { ' ' } else { c })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let clean: String = clean.chars().take(80).collect();
    if clean.is_empty() { fallback.to_string() } else { clean }
}

/// `when` es la fecha ya formateada por la interfaz (p. ej. "27 oct 2026 · 10:30").
#[tauri::command]
pub fn export_session(app: AppHandle, id: String, format: String, when: String) -> Result<String, String> {
    let state = app.state::<AppState>();
    let (kind, title, duration_ms, acta): (String, String, i64, String) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.query_row("SELECT kind, title, duration_ms, acta FROM sessions WHERE id=?1", params![id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .map_err(|_| "La grabación ya no existe".to_string())?
    };
    let tr = std::fs::read_to_string(rec::session_dir(&state, &id).join("transcripcion.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<transcribe::Transcript>(&raw).ok());
    if tr.is_none() && acta.is_empty() {
        return Err("No hay nada que exportar: primero transcribe la grabación".into());
    }
    let b = blocks(&kind, &title, &when, duration_ms, &acta, tr);

    let what = if kind == "capacitacion" { "Capacitación" } else { "Reunión" };
    let ext = if format == "docx" { "docx" } else { "md" };
    let path = out_path(&app, &title, what, ext)?;
    if ext == "docx" {
        docx(&b, &path)?;
    } else {
        std::fs::write(&path, markdown(&b)).map_err(|e| e.to_string())?;
    }
    reveal(&path);
    Ok(path.to_string_lossy().into_owned())
}

/// Ruta libre en Documentos\Lienzo: no sobrescribe, si ya existe agrega (2), (3)…
pub(crate) fn out_path(app: &AppHandle, title: &str, fallback: &str, ext: &str) -> Result<PathBuf, String> {
    let dir = app.path().document_dir().map_err(|e| e.to_string())?.join("Lienzo");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let base = file_name(title, fallback);
    let mut path = dir.join(format!("{base}.{ext}"));
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{base} ({n}).{ext}"));
        n += 1;
    }
    Ok(path)
}

/// Abre el Explorador con el archivo seleccionado.
pub(crate) fn reveal(path: &std::path::Path) {
    let _ = std::process::Command::new("explorer").arg(format!("/select,{}", path.display())).spawn();
}
