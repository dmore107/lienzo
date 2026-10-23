// Modo portable: si junto a Lienzo.exe hay una carpeta "datos", todo vive ahí
// (base, imágenes, grabaciones, modelos y los datos de la ventana) y no se toca el registro.
// El mismo ejecutable funciona instalado o portable; solo cambia dónde guarda.
use std::path::PathBuf;

use base64::Engine;
use tauri::AppHandle;

/// Carpeta de datos portable, si existe junto al ejecutable.
pub fn data_dir() -> Option<PathBuf> {
    let dir = std::env::current_exe().ok()?.parent()?.join("datos");
    dir.is_dir().then_some(dir)
}

/// Borra el ejecutable anterior que dejó la última actualización (Windows no deja borrarlo mientras corre).
pub fn cleanup_old() {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::fs::remove_file(exe.with_extension("exe.old"));
    }
}

fn b64_text(s: &str) -> Option<String> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(s.trim()).ok()?;
    String::from_utf8(bytes).ok()
}

/// Verifica la firma con la misma llave pública del actualizador y que corresponda a este archivo.
fn verify(app: &AppHandle, data: &[u8], sig_b64: &str, file: &str) -> Result<(), String> {
    let pk = app
        .config()
        .plugins
        .0
        .get("updater")
        .and_then(|u| u.get("pubkey"))
        .and_then(|v| v.as_str())
        .and_then(b64_text)
        .ok_or("Falta la llave pública de actualización")?;
    let pk = minisign_verify::PublicKey::decode(&pk).map_err(|e| e.to_string())?;
    let sig = minisign_verify::Signature::decode(&b64_text(sig_b64).ok_or("Firma ilegible")?).map_err(|e| e.to_string())?;
    pk.verify(data, &sig, true).map_err(|_| "La descarga no tiene una firma válida".to_string())?;
    // El comentario firmado incluye el nombre del archivo: evita que se cuele otra versión firmada
    if !sig.trusted_comment().split('\t').any(|p| p == format!("file:{file}")) {
        return Err("La firma no corresponde a esta versión".into());
    }
    Ok(())
}

/// Descarga el ejecutable portable de la versión indicada, verifica su firma y lo cambia por el actual.
pub async fn update(app: &AppHandle, version: &str) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let file = format!("Lienzo_{version}_portable.exe");
    let url = format!("https://github.com/dmore107/lienzo/releases/download/v{version}/{file}");
    let get = |u: String| async move { reqwest::get(u).await.and_then(|r| r.error_for_status()) };
    let bin = get(url.clone())
        .await
        .map_err(|e| format!("No se pudo descargar la actualización: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("La descarga se interrumpió: {e}"))?;
    let sig = get(format!("{url}.sig")).await.map_err(|e| e.to_string())?.text().await.map_err(|e| e.to_string())?;
    verify(app, &bin, &sig, &file)?;

    // Windows deja renombrar un ejecutable en uso, pero no sobrescribirlo
    let new = exe.with_extension("exe.new");
    let old = exe.with_extension("exe.old");
    std::fs::write(&new, &bin).map_err(|e| format!("No se pudo escribir junto a Lienzo ({e}). ¿La carpeta es de solo lectura?"))?;
    let _ = std::fs::remove_file(&old);
    std::fs::rename(&exe, &old).map_err(|e| e.to_string())?;
    if let Err(e) = std::fs::rename(&new, &exe) {
        let _ = std::fs::rename(&old, &exe);
        return Err(e.to_string());
    }
    // Si se estaba grabando, se guarda antes de reiniciar
    let _ = crate::rec::stop(app);
    app.request_restart();
    Ok(())
}
