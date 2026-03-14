use crate::AppSettings;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "settings.json";

pub fn load(app: &AppHandle) -> Result<AppSettings, String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    let settings = AppSettings {
        token: store
            .get("token")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default(),
        refresh_minutes: store
            .get("refresh_minutes")
            .and_then(|v| v.as_u64())
            .unwrap_or(5),
        show_percent_text: store
            .get("show_percent_text")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        theme: store
            .get("theme")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "system".to_string()),
        alert_threshold: store
            .get("alert_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(20.0),
        auto_start: store
            .get("auto_start")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    };
    Ok(settings)
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set("token", serde_json::json!(settings.token));
    store.set("refresh_minutes", serde_json::json!(settings.refresh_minutes));
    store.set("show_percent_text", serde_json::json!(settings.show_percent_text));
    store.set("theme", serde_json::json!(settings.theme));
    store.set("alert_threshold", serde_json::json!(settings.alert_threshold));
    store.set("auto_start", serde_json::json!(settings.auto_start));
    store.save().map_err(|e| e.to_string())
}
