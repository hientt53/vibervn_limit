# Architecture

**Analysis Date:** 2026-03-17

## Pattern Overview

**Overall:** Tauri desktop application — thin Rust backend with vanilla JS frontend communicating via IPC commands and events.

**Key Characteristics:**
- Rust backend owns all state, business logic, and external API calls
- Frontend is a static HTML/JS/CSS bundle with no build step or framework
- Communication is strictly one-way invocation (JS → Rust via `invoke`) and event push (Rust → JS via `emit`)
- App runs as a macOS menu bar / system tray agent with no dock icon

## Layers

**Backend Core (`src-tauri/src/lib.rs`):**
- Purpose: Application entry point, state management, Tauri command handlers, tray setup, background loop
- Location: `src-tauri/src/lib.rs`
- Contains: `AppState`, `AppSettings`, `BalanceInfo` structs; all `#[tauri::command]` handlers; `run()` bootstrap
- Depends on: `api`, `icon`, `store` modules
- Used by: `main.rs` (calls `tauri_app_lib::run()`)

**API Layer (`src-tauri/src/api.rs`):**
- Purpose: All HTTP calls to the external Viber balance API
- Location: `src-tauri/src/api.rs`
- Contains: `fetch_balance`, `fetch_logs`, `fetch_log_stats`; response deserialization structs; `LogsPage`, `LogItem`, `LogStatItem` public types
- Depends on: `reqwest`, `serde_json`, `BalanceInfo` from `lib.rs`
- Used by: `lib.rs` command handlers and background refresh loop

**Persistence Layer (`src-tauri/src/store.rs`):**
- Purpose: Read/write `AppSettings` to disk via `tauri-plugin-store`
- Location: `src-tauri/src/store.rs`
- Contains: `load(app)` and `save(app, settings)` functions; store file constant `settings.json`
- Depends on: `tauri_plugin_store::StoreExt`, `AppSettings`
- Used by: `lib.rs` on startup and on `save_settings` command

**Icon Renderer (`src-tauri/src/icon.rs`):**
- Purpose: Procedurally generate RGBA battery icon pixels for the system tray
- Location: `src-tauri/src/icon.rs`
- Contains: `generate_battery_icon(percent, width, height)` — pure pixel math with subpixel anti-aliasing
- Depends on: nothing (pure Rust, no external crates)
- Used by: `lib.rs` `update_tray()` and initial tray setup

**Frontend (`src/`):**
- Purpose: Popup window UI — balance display, logs browser, settings form
- Location: `src/index.html`, `src/popup.js`, `src/styles.css`
- Contains: Three tab views (Balance, Logs, Settings) in a single HTML page; all UI logic in `popup.js`
- Depends on: `window.__TAURI__` global injected by Tauri runtime
- Used by: Tauri webview window labeled `"popup"`

## Data Flow

**Balance Refresh (background):**

1. `spawn_refresh_loop` in `lib.rs` runs a `tokio` interval loop
2. Reads `token` and `refresh_minutes` from `AppState.settings` (Mutex)
3. Calls `api::fetch_balance(&token)` — HTTP GET to `https://viber.claudegateway.site/api/balance/check`
4. On success: updates `AppState.last_balance`, calls `update_tray()` to redraw icon, emits `"balance-updated"` event to frontend
5. On error: emits `"balance-error"` event, sets tray to `"⚠ Error"`
6. Checks alert threshold; fires OS notification via `tauri-plugin-notification` if balance is low

**Frontend Balance Display:**

1. `DOMContentLoaded` → `loadBalance()` invokes `get_balance` command → reads `AppState.last_balance`
2. Listens for `"balance-updated"` event → calls `updateUI(payload)` to update ring, stats, countdown
3. Refresh button invokes `refresh_now` → backend emits `"refresh-requested"` (loop picks it up on next tick)

**Settings Save:**

1. User clicks "Save & Apply" → JS invokes `save_settings` with new settings object
2. Backend updates `AppState.settings` Mutex, calls `store::save()` to persist to `settings.json`
3. Emits `"settings-changed"` event; if token is set, immediately spawns a one-shot balance fetch
4. Frontend listens for `"settings-changed"` → re-applies theme and reloads balance display

**Logs Flow:**

1. JS invokes `get_logs` with pagination/filter params → backend calls `api::fetch_logs`
2. JS invokes `get_log_stats` → backend calls `api::fetch_log_stats`
3. Results rendered directly into DOM; no caching on frontend

**State Management:**
- All mutable state lives in `AppState` behind `Mutex<T>` fields, wrapped in `Arc` for cross-thread sharing
- Frontend holds no persistent state except `logState` (pagination/filter object) and `window._nextResetTime`

## Key Abstractions

**`AppState`:**
- Purpose: Single shared state container for the running app
- Location: `src-tauri/src/lib.rs` (lines 55–59)
- Pattern: `Arc<AppState>` managed by Tauri; fields are `Mutex<T>` for interior mutability

**`BalanceInfo`:**
- Purpose: Normalized balance data passed between backend and frontend
- Location: `src-tauri/src/lib.rs` (lines 16–24)
- Pattern: `Serialize + Deserialize` — serialized to JSON for IPC events and command responses

**`AppSettings`:**
- Purpose: All user-configurable settings; persisted to and loaded from store
- Location: `src-tauri/src/lib.rs` (lines 27–53)
- Pattern: `Default` impl provides fallback values; `serde(default)` on newer fields for forward compatibility

**Tauri Commands:**
- Purpose: IPC bridge — JS calls these via `window.__TAURI__.core.invoke('command_name', args)`
- Location: `src-tauri/src/lib.rs` — registered in `invoke_handler!` macro
- Commands: `get_balance`, `get_settings`, `save_settings`, `refresh_now`, `hide_popup`, `toggle_autostart`, `get_logs`, `get_log_stats`, `export_logs_csv`, `get_daily_stats`

## Entry Points

**Binary Entry (`src-tauri/src/main.rs`):**
- Location: `src-tauri/src/main.rs`
- Triggers: OS process start
- Responsibilities: Calls `tauri_app_lib::run()` — nothing else

**Library Entry (`src-tauri/src/lib.rs` → `run()`):**
- Location: `src-tauri/src/lib.rs` (line 355)
- Triggers: Called by `main.rs`
- Responsibilities: Registers plugins, loads settings from store, builds tray icon and menu, spawns background refresh loop, registers all Tauri commands, starts event loop

**Frontend Entry (`src/index.html`):**
- Location: `src/index.html`
- Triggers: Tauri webview window `"popup"` loads `index.html`
- Responsibilities: Renders tab UI, loads `popup.js`

**Frontend Init (`src/popup.js` → `DOMContentLoaded`):**
- Location: `src/popup.js` (line 603)
- Triggers: DOM ready after Tauri runtime available
- Responsibilities: Applies theme, loads initial balance, sets up event listeners for backend events

## Error Handling

**Strategy:** Errors propagate as `Result<T, String>` from Tauri commands; frontend catches via try/catch on `invoke` promises.

**Patterns:**
- Backend: `map_err(|e| e.to_string())` converts typed errors to strings for IPC transport
- HTTP errors: checked by status code before JSON parsing; 401/403 returns specific auth error message
- Frontend: `console.error` + DOM text update (e.g., `last-update` element shows error string)
- Background loop: logs to `eprintln!`, updates tray to `"⚠ Error"`, emits `"balance-error"` event

## Cross-Cutting Concerns

**Logging:** `eprintln!` to stderr in backend; `console.error` in frontend. No structured logging framework.
**Validation:** Input validation in frontend only (blur handlers clamp numeric inputs). Backend trusts frontend-provided settings values.
**Authentication:** Bearer token stored in `tauri-plugin-store` (encrypted app data directory). Sent as `Authorization: Bearer {token}` header on every API request.

---

*Architecture analysis: 2026-03-17*
