// Acta de una reunión o capacitación generada con Gemini a partir de la transcripción.
//
// Solo se envía el texto transcrito (nunca el audio) y solo cuando lo pides. La clave de
// Gemini vive en el Administrador de credenciales de Windows, no en archivos de Lienzo.
use std::time::Duration;

use rusqlite::params;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

use crate::{rec, AppState};

/// Si Google retira el primero, se intenta con el alias que siempre apunta al Flash vigente.
const MODELS: [&str; 2] = ["gemini-2.5-flash", "gemini-flash-latest"];
const API: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/* ===================== Clave en el Administrador de credenciales ===================== */

#[cfg(windows)]
mod cred {
    use windows_sys::Win32::Security::Credentials::{
        CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
    };

    const TARGET: &str = "Lienzo/gemini";

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn get() -> Option<String> {
        unsafe {
            let mut p: *mut CREDENTIALW = std::ptr::null_mut();
            if CredReadW(wide(TARGET).as_ptr(), CRED_TYPE_GENERIC, 0, &mut p) == 0 || p.is_null() {
                return None;
            }
            let c = &*p;
            let v = if c.CredentialBlob.is_null() {
                None
            } else {
                String::from_utf8(std::slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize).to_vec()).ok()
            };
            CredFree(p as *const _);
            v.filter(|k| !k.is_empty())
        }
    }

    pub fn set(key: &str) -> Result<(), String> {
        let (mut target, mut user, mut blob) = (wide(TARGET), wide("gemini"), key.as_bytes().to_vec());
        unsafe {
            let mut c: CREDENTIALW = std::mem::zeroed();
            c.Type = CRED_TYPE_GENERIC;
            c.TargetName = target.as_mut_ptr();
            c.UserName = user.as_mut_ptr();
            c.CredentialBlobSize = blob.len() as u32;
            c.CredentialBlob = blob.as_mut_ptr();
            c.Persist = CRED_PERSIST_LOCAL_MACHINE;
            if CredWriteW(&c, 0) == 0 {
                return Err(format!("No se pudo guardar la clave: {}", std::io::Error::last_os_error()));
            }
        }
        Ok(())
    }

    pub fn delete() {
        unsafe { CredDeleteW(wide(TARGET).as_ptr(), CRED_TYPE_GENERIC, 0) };
    }
}

#[cfg(not(windows))]
mod cred {
    pub fn get() -> Option<String> { None }
    pub fn set(_: &str) -> Result<(), String> { Err("Solo disponible en Windows".into()) }
    pub fn delete() {}
}

#[tauri::command]
pub fn gemini_status() -> bool {
    cred::get().is_some()
}

/// Comprueba la clave con Google antes de guardarla, para avisar de inmediato si está mal copiada.
#[tauri::command]
pub async fn set_gemini_key(key: String) -> Result<(), String> {
    let key = key.trim().to_string();
    if key.len() < 20 || key.contains(char::is_whitespace) {
        return Err("Esa no parece una clave de Gemini".into());
    }
    let resp = client()?
        .get(format!("{API}?pageSize=1"))
        .header("x-goog-api-key", &key)
        .send()
        .await
        .map_err(|e| format!("No se pudo conectar con Google: {e}"))?;
    if !resp.status().is_success() {
        return Err(match resp.status().as_u16() {
            400 | 401 | 403 => "Google no aceptó la clave: revisa que la copiaste completa".into(),
            s => format!("Google respondió con un error ({s}); intenta de nuevo"),
        });
    }
    cred::set(&key)
}

#[tauri::command]
pub fn delete_gemini_key() {
    cred::delete();
}

/* ===================== Generar el acta ===================== */

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder().timeout(Duration::from_secs(180)).build().map_err(|e| e.to_string())
}

fn schema(kind: &str) -> Value {
    let s = json!({ "type": "STRING" });
    let list = |v: Value| json!({ "type": "ARRAY", "items": v });
    let obj = |props: Value, req: &[&str]| json!({ "type": "OBJECT", "properties": props, "required": req });
    let temas = list(obj(json!({ "tema": s, "minuto": s }), &["tema", "minuto"]));
    if kind == "capacitacion" {
        obj(
            json!({
                "resumen": s,
                "conceptos": list(obj(json!({ "concepto": s, "explicacion": s }), &["concepto", "explicacion"])),
                "pasos": list(s.clone()),
                "temas": temas,
            }),
            &["resumen", "conceptos", "pasos", "temas"],
        )
    } else {
        obj(
            json!({
                "resumen": s,
                "decisiones": list(s.clone()),
                "tareas": list(obj(json!({ "tarea": s, "responsable": s, "fecha": s }), &["tarea", "responsable", "fecha"])),
                "temas": temas,
            }),
            &["resumen", "decisiones", "tareas", "temas"],
        )
    }
}

fn prompt(kind: &str, title: &str, transcript: &str) -> String {
    let what = if kind == "capacitacion" { "capacitación" } else { "reunión de trabajo" };
    let title = if title.is_empty() { String::new() } else { format!(" titulada «{title}»") };
    let fields = if kind == "capacitacion" {
        "- resumen: 3 a 5 frases con lo esencial de la capacitación.\n\
         - conceptos: las ideas o términos clave, cada uno con una explicación breve y clara.\n\
         - pasos: los procedimientos que se enseñaron, en orden, cada paso como una acción concreta.\n"
    } else {
        "- resumen: 3 a 5 frases con lo esencial de la reunión.\n\
         - decisiones: lo que quedó decidido o acordado, una por elemento.\n\
         - tareas: compromisos concretos. «responsable» es quien la asumió (\"Tú\" si fue la persona que grabó, \
           un nombre si se menciona, o \"\" si no queda claro). «fecha» es el plazo tal como se dijo, o \"\".\n"
    };
    format!(
        "Eres un asistente que redacta actas en español claro y conciso.\n\
         A continuación está la transcripción automática de una {what}{title}. Cada línea empieza con el \
         minuto entre corchetes; «Tú» es quien grabó y «Participantes» son las demás personas.\n\
         La transcripción puede tener errores de reconocimiento: interprétalos por contexto, pero no inventes \
         nada que no se haya dicho. Si una sección no aplica, devuélvela vacía.\n\n\
         Devuelve:\n{fields}\
         - temas: los temas tratados en orden, con el minuto (mm:ss o h:mm:ss) en que empieza cada uno, \
           tomado de la transcripción.\n\n\
         Transcripción:\n{transcript}"
    )
}

/// Pide a Gemini una respuesta JSON con el esquema dado (también la usan el informe y el tutorial).
/// Devuelve el JSON tal como llegó, ya validado.
pub(crate) async fn gemini(prompt: String, schema: Value) -> Result<String, String> {
    let key = cred::get().ok_or("Primero configura tu clave de Gemini")?;
    ask(&key, prompt, schema).await
}

async fn ask(key: &str, prompt: String, schema: Value) -> Result<String, String> {
    let body = json!({
        "contents": [{ "role": "user", "parts": [{ "text": prompt }] }],
        "generationConfig": {
            "temperature": 0.2,
            "responseMimeType": "application/json",
            "responseSchema": schema,
        },
    });
    let body = serde_json::to_vec(&body).map_err(|e| e.to_string())?;
    let http = client()?;
    for (i, model) in MODELS.iter().enumerate() {
        let resp = http
            .post(format!("{API}/{model}:generateContent"))
            .header("x-goog-api-key", key)
            .header("content-type", "application/json")
            .body(body.clone())
            .send()
            .await
            .map_err(|e| format!("No se pudo conectar con Gemini: {e}"))?;
        let status = resp.status().as_u16();
        let raw = resp.bytes().await.map_err(|e| e.to_string())?;
        let v: Value = serde_json::from_slice(&raw).unwrap_or(Value::Null);
        if status == 404 && i + 1 < MODELS.len() {
            continue;
        }
        if status != 200 {
            let msg = v["error"]["message"].as_str().unwrap_or("");
            return Err(match status {
                400 if msg.contains("API key") => "La clave de Gemini ya no es válida; vuelve a configurarla".into(),
                401 | 403 => "La clave de Gemini no tiene permiso; vuelve a configurarla".into(),
                429 => "Llegaste al límite gratuito de Gemini por ahora; intenta de nuevo en un rato".into(),
                _ => format!("Gemini respondió con un error ({status}). {msg}").trim().to_string(),
            });
        }
        let text = v["candidates"][0]["content"]["parts"]
            .as_array()
            .map(|parts| parts.iter().filter_map(|p| p["text"].as_str()).collect::<String>())
            .unwrap_or_default();
        if text.is_empty() {
            let why = v["candidates"][0]["finishReason"].as_str().or(v["promptFeedback"]["blockReason"].as_str()).unwrap_or("sin respuesta");
            return Err(format!("Gemini no devolvió una respuesta ({why})"));
        }
        let acta: Value = serde_json::from_str(&text).map_err(|_| "Gemini devolvió una respuesta incompleta; intenta de nuevo".to_string())?;
        return serde_json::to_string(&acta).map_err(|e| e.to_string());
    }
    Err("Ningún modelo de Gemini está disponible".into())
}

#[tauri::command]
pub async fn generate_acta(app: AppHandle, id: String) -> Result<String, String> {
    let key = cred::get().ok_or("Primero configura tu clave de Gemini")?;
    let (kind, title, status, transcript): (String, String, String, String) = {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.query_row("SELECT kind, title, status, transcript FROM sessions WHERE id=?1", params![id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .map_err(|_| "La grabación ya no existe".to_string())?
    };
    if status != "transcrita" || transcript.trim().is_empty() {
        return Err("Primero transcribe la grabación".into());
    }
    let acta = ask(&key, prompt(&kind, &title, &transcript), schema(&kind)).await?;
    {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute("UPDATE sessions SET acta=?1 WHERE id=?2", params![acta, id]).map_err(|e| e.to_string())?;
    }
    rec::emit_session(&app, &id);
    Ok(acta)
}
