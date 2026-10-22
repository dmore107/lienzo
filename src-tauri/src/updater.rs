// Actualizaciones desde GitHub Releases (firmadas; la llave pública va en tauri.conf.json).
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    version: String,
    current: String,
    notes: Option<String>,
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    let up = app.updater().map_err(|e| e.to_string())?.check().await.map_err(|e| e.to_string())?;
    Ok(up.map(|u| UpdateInfo { version: u.version.clone(), current: u.current_version.clone(), notes: u.body.clone() }))
}

/// Descarga, verifica la firma e instala. En Windows el instalador cierra Lienzo y lo vuelve a abrir.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let Some(u) = app.updater().map_err(|e| e.to_string())?.check().await.map_err(|e| e.to_string())? else {
        return Err("Ya tienes la última versión".into());
    };
    u.download_and_install(|_, _| {}, || {}).await.map_err(|e| e.to_string())?;
    app.restart();
}
