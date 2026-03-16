use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

mod api;
mod icon;
mod store;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub percent: f64,           // remaining %
    pub used_usd: f64,
    pub remain_usd: f64,
    pub total_usd: f64,
    pub unlimited: bool,
    pub next_reset_time: Option<i64>,
    pub expired_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub token: String,
    pub refresh_minutes: u64,
    pub show_percent_text: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_threshold")]
    pub alert_threshold: f64,
    #[serde(default)]
    pub auto_start: bool,
}

fn default_theme() -> String { "system".to_string() }
fn default_threshold() -> f64 { 20.0 }

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            token: String::new(),
            refresh_minutes: 5,
            show_percent_text: true,
            theme: default_theme(),
            alert_threshold: default_threshold(),
            auto_start: false,
        }
    }
}

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub last_balance: Mutex<Option<BalanceInfo>>,
    pub alerted: Mutex<bool>,
}

// --- Tauri commands exposed to frontend ---

#[tauri::command]
async fn get_balance(state: tauri::State<'_, Arc<AppState>>) -> Result<Option<BalanceInfo>, String> {
    Ok(state.last_balance.lock().unwrap().clone())
}

#[tauri::command]
async fn get_settings(state: tauri::State<'_, Arc<AppState>>) -> Result<AppSettings, String> {
    Ok(state.settings.lock().unwrap().clone())
}

#[tauri::command]
async fn save_settings(
    new_settings: AppSettings,
    state: tauri::State<'_, Arc<AppState>>,
    app: AppHandle,
) -> Result<(), String> {
    {
        let mut s = state.settings.lock().unwrap();
        *s = new_settings.clone();
    }
    store::save(&app, &new_settings).map_err(|e| e.to_string())?;
    app.emit("settings-changed", ()).ok();

    // Fetch balance immediately if token is set
    let token = new_settings.token.clone();
    if !token.is_empty() {
        let state_clone = state.inner().clone();
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            let show_text = state_clone.settings.lock().unwrap().show_percent_text;
            match api::fetch_balance(&token).await {
                Ok(balance) => {
                    let label = format_label(&balance, show_text);
                    update_tray(&app_clone, Some(balance.percent), show_text, &label);
                    *state_clone.last_balance.lock().unwrap() = Some(balance.clone());
                    app_clone.emit("balance-updated", balance).ok();
                }
                Err(e) => {
                    eprintln!("API error after settings save: {e}");
                    update_tray(&app_clone, None, show_text, "⚠ Error");
                    app_clone.emit("balance-error", e.to_string()).ok();
                }
            }
        });
    }

    Ok(())
}

#[tauri::command]
async fn refresh_now(app: AppHandle) -> Result<(), String> {
    app.emit("refresh-requested", ()).ok();
    Ok(())
}

#[tauri::command]
async fn hide_popup(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("popup") {
        w.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn toggle_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let autostart = app.autolaunch();
    if enabled {
        autostart.enable().map_err(|e| e.to_string())?;
    } else {
        autostart.disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn export_logs_csv(
    state: tauri::State<'_, Arc<AppState>>,
    log_type: i32,
    model_name: Option<String>,
    start_timestamp: Option<i64>,
    end_timestamp: Option<i64>,
) -> Result<String, String> {
    let token = state.settings.lock().unwrap().token.clone();
    if token.is_empty() {
        return Err("No token configured".to_string());
    }
    // Fetch all logs (up to 1000)
    let logs = api::fetch_logs(
        &token, 0, 1000, log_type,
        model_name.as_deref(), start_timestamp, end_timestamp,
    ).await?;
    // Build CSV
    let mut csv = String::from("Time,Model,Tokens,Cost\n");
    for item in &logs.items {
        let tokens = item.prompt_tokens + item.completion_tokens;
        let cost = item.quota as f64 / 500000.0;
        csv.push_str(&format!("{},{},{},{:.6}\n", item.created_at, item.model_name, tokens, cost));
    }
    Ok(csv)
}

#[tauri::command]
async fn get_daily_stats(
    state: tauri::State<'_, Arc<AppState>>,
    days: i32,
) -> Result<serde_json::Value, String> {
    let token = state.settings.lock().unwrap().token.clone();
    if token.is_empty() {
        return Err("No token configured".to_string());
    }
    // Use log stats to get daily breakdown
    let now = chrono::Utc::now().timestamp();
    let start = now - (days as i64 * 86400);
    api::fetch_log_stats(&token, 0, None, Some(start), Some(now)).await
}

#[tauri::command]
async fn get_logs(
    state: tauri::State<'_, Arc<AppState>>,
    page: i32,
    page_size: i32,
    log_type: i32,
    model_name: Option<String>,
    start_timestamp: Option<i64>,
    end_timestamp: Option<i64>,
) -> Result<api::LogsPage, String> {
    let token = state.settings.lock().unwrap().token.clone();
    if token.is_empty() {
        return Err("No token configured".to_string());
    }
    api::fetch_logs(
        &token,
        page,
        page_size,
        log_type,
        model_name.as_deref(),
        start_timestamp,
        end_timestamp,
    )
    .await
}

#[tauri::command]
async fn get_log_stats(
    state: tauri::State<'_, Arc<AppState>>,
    log_type: i32,
    model_name: Option<String>,
    start_timestamp: Option<i64>,
    end_timestamp: Option<i64>,
) -> Result<serde_json::Value, String> {
    let token = state.settings.lock().unwrap().token.clone();
    if token.is_empty() {
        return Err("No token configured".to_string());
    }
    api::fetch_log_stats(
        &token,
        log_type,
        model_name.as_deref(),
        start_timestamp,
        end_timestamp,
    )
    .await
}

fn open_or_focus_window(app: &AppHandle, label: &str, url: &str, title: &str, w: u32, h: u32) {
    if let Some(win) = app.get_webview_window(label) {
        win.show().ok();
        win.set_focus().ok();
    } else {
        let win = tauri::WebviewWindowBuilder::new(app, label, tauri::WebviewUrl::App(url.into()))
            .title(title)
            .inner_size(w as f64, h as f64)
            .resizable(true)
            .decorations(true)
            .transparent(false)
            .build();

        if let Ok(w) = win {
            // macOS: native vibrancy blur
            #[cfg(target_os = "macos")]
            {
                use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
                apply_vibrancy(&w, NSVisualEffectMaterial::Popover, None, None).ok();
            }
        }
    }
}

// --- Background refresh loop ---

fn spawn_refresh_loop(app: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        let mut ticker = interval(Duration::from_secs(10)); // initial fast check
        let mut current_interval_mins = 0u64;

        loop {
            ticker.tick().await;

            let (token, refresh_mins, show_text) = {
                let s = state.settings.lock().unwrap();
                (s.token.clone(), s.refresh_minutes, s.show_percent_text)
            };

            // Re-schedule if interval changed
            if refresh_mins != current_interval_mins {
                current_interval_mins = refresh_mins;
                ticker = interval(Duration::from_secs(refresh_mins * 60));
                ticker.tick().await; // consume immediate tick from reset
            }

            if token.is_empty() {
                update_tray(&app, None, show_text, "⚙ Set Token");
                continue;
            }

            match api::fetch_balance(&token).await {
                Ok(balance) => {
                    let label = format_label(&balance, show_text);
                    update_tray(&app, Some(balance.percent), show_text, &label);
                    *state.last_balance.lock().unwrap() = Some(balance.clone());

                    // Low balance notification
                    if !balance.unlimited {
                        let threshold = state.settings.lock().unwrap().alert_threshold;
                        let mut alerted = state.alerted.lock().unwrap();
                        if balance.percent <= threshold && !*alerted {
                            *alerted = true;
                            use tauri_plugin_notification::NotificationExt;
                            app.notification()
                                .builder()
                                .title("Viber Balance Low")
                                .body(format!("Balance is {:.0}% — ${:.2} remaining", balance.percent, balance.remain_usd))
                                .show()
                                .ok();
                        } else if balance.percent > threshold {
                            *alerted = false;
                        }
                    }

                    app.emit("balance-updated", balance).ok();
                }
                Err(e) => {
                    eprintln!("API error: {e}");
                    update_tray(&app, None, show_text, "⚠ Error");
                    app.emit("balance-error", e.to_string()).ok();
                }
            }
        }
    });
}

fn format_label(balance: &BalanceInfo, show_text: bool) -> String {
    if balance.unlimited {
        return if show_text { "∞".to_string() } else { String::new() };
    }
    if show_text {
        format!("{:.0}%", balance.percent)
    } else {
        String::new()
    }
}

/// Returns the platform-appropriate tray icon dimensions.
fn tray_icon_size() -> u32 {
    #[cfg(target_os = "windows")]
    { 16 }
    #[cfg(not(target_os = "windows"))]
    { 22 }
}

fn update_tray(app: &AppHandle, percent: Option<f64>, _show_text: bool, text: &str) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        // macOS: show text next to icon in menu bar
        #[cfg(target_os = "macos")]
        tray.set_title(Some(text)).ok();

        // Windows/Linux: show text as tooltip on hover
        #[cfg(not(target_os = "macos"))]
        tray.set_tooltip(Some(text)).ok();

        let size = tray_icon_size();
        if let Some(pct) = percent {
            if let Ok(img_data) = icon::generate_battery_icon(pct, size, size) {
                if let Ok(img) = Ok::<_, ()>(Image::new_owned(img_data, size, size)) {
                    tray.set_icon(Some(img)).ok();
                }
            }
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            // Load settings
            let settings = store::load(app.handle()).unwrap_or_default();
            let state = Arc::new(AppState {
                settings: Mutex::new(settings),
                last_balance: Mutex::new(None),
                alerted: Mutex::new(false),
            });
            app.manage(state.clone());

            // Build tray
            let tray_menu = Menu::with_items(app, &[
                &MenuItem::with_id(app, "open", "Open", true, None::<&str>)?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
            ])?;

            let icon_size = tray_icon_size();
            let initial_icon = icon::generate_battery_icon(-1.0, icon_size, icon_size).unwrap_or_default();
            let img = Image::new_owned(initial_icon, icon_size, icon_size);

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(img)
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "open" => { open_or_focus_window(app, "popup", "index.html", "Viber Balance", 640, 620); }
                    "settings" => { open_or_focus_window(app, "popup", "index.html", "Viber Balance", 640, 620); }
                    "quit" => { app.exit(0); }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        open_or_focus_window(app, "popup", "index.html", "Viber Balance", 640, 620);
                    }
                })
                .build(app)?;

            // Spawn background loop
            let handle = app.handle().clone();
            spawn_refresh_loop(handle, state);

            // macOS: hide from dock (pure tray app)
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_balance,
            get_settings,
            save_settings,
            refresh_now,
            hide_popup,
            toggle_autostart,
            get_logs,
            get_log_stats,
            export_logs_csv,
            get_daily_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
