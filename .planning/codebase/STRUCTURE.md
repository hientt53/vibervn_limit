# Codebase Structure

**Analysis Date:** 2026-03-17

## Directory Layout

```
vibervn_limit/
├── src/                    # Frontend (static HTML/JS/CSS — no build step)
│   ├── index.html          # Single popup window HTML (all three tabs)
│   ├── popup.js            # All frontend logic (~624 lines)
│   └── styles.css          # All styles (~15KB)
├── src-tauri/              # Rust backend (Tauri app)
│   ├── src/                # Rust source files
│   │   ├── main.rs         # Binary entry point (calls lib::run)
│   │   ├── lib.rs          # Core: state, commands, tray, background loop
│   │   ├── api.rs          # HTTP client for Viber balance API
│   │   ├── store.rs        # Settings persistence via tauri-plugin-store
│   │   └── icon.rs         # Procedural battery icon pixel renderer
│   ├── capabilities/
│   │   └── default.json    # Tauri permission grants for webview windows
│   ├── icons/              # App bundle icons (PNG, ICNS, ICO)
│   ├── gen/                # Tauri-generated schemas (do not edit)
│   ├── Cargo.toml          # Rust dependencies
│   ├── Cargo.lock          # Locked dependency versions
│   ├── build.rs            # Tauri build script
│   └── tauri.conf.json     # Tauri app configuration
├── .planning/              # GSD planning documents
│   └── codebase/           # Codebase analysis docs
├── .github/                # GitHub Actions workflows
├── package.json            # Node manifest (only @tauri-apps/cli devDep)
└── README.md               # Project documentation
```

## Directory Purposes

**`src/`:**
- Purpose: Frontend webview content served to the Tauri popup window
- Contains: One HTML file, one JS file, one CSS file — no bundler, no framework
- Key files: `src/index.html` (markup), `src/popup.js` (all logic), `src/styles.css` (all styles)

**`src-tauri/src/`:**
- Purpose: All Rust application code
- Contains: 5 `.rs` files — entry point, core logic, API client, persistence, icon renderer
- Key files: `src-tauri/src/lib.rs` (main logic hub), `src-tauri/src/api.rs` (external calls)

**`src-tauri/capabilities/`:**
- Purpose: Tauri v2 capability definitions — controls which Tauri APIs the webview can access
- Contains: `default.json` — grants permissions for store, notification, dialog, autostart
- Generated: No (hand-maintained)
- Committed: Yes

**`src-tauri/gen/`:**
- Purpose: Tauri-generated JSON schemas for capability validation
- Generated: Yes (by `tauri build` / `tauri dev`)
- Committed: Yes (schemas only, not build artifacts)

**`src-tauri/icons/`:**
- Purpose: App bundle icons for all platforms and sizes
- Generated: No (static assets)
- Committed: Yes

**`src-tauri/target/`:**
- Purpose: Rust build output
- Generated: Yes
- Committed: No (in `.gitignore`)

## Key File Locations

**Entry Points:**
- `src-tauri/src/main.rs`: Binary entry — calls `tauri_app_lib::run()`
- `src-tauri/src/lib.rs`: `run()` function — full app bootstrap
- `src/index.html`: Webview entry — loaded by Tauri popup window
- `src/popup.js`: Frontend init — `DOMContentLoaded` listener at line 603

**Configuration:**
- `src-tauri/tauri.conf.json`: App identity, window config, bundle targets, frontend dist path
- `src-tauri/Cargo.toml`: Rust dependencies and crate metadata
- `src-tauri/capabilities/default.json`: Tauri API permission grants
- `package.json`: Node manifest (only used for `@tauri-apps/cli`)

**Core Logic:**
- `src-tauri/src/lib.rs`: State structs, all Tauri commands, tray setup, background refresh loop
- `src-tauri/src/api.rs`: All HTTP calls to `viber.claudegateway.site` API
- `src-tauri/src/store.rs`: Settings load/save via `tauri-plugin-store`
- `src-tauri/src/icon.rs`: Battery icon pixel generation

**Frontend Logic:**
- `src/popup.js`: Balance display, logs browser, settings form, tab switching, event listeners
- `src/styles.css`: All visual styles including theme variables and responsive layout

## Naming Conventions

**Files:**
- Rust modules: `snake_case.rs` (e.g., `api.rs`, `store.rs`, `icon.rs`)
- Frontend: lowercase with no separator (e.g., `popup.js`, `styles.css`, `index.html`)

**Rust Identifiers:**
- Structs: `PascalCase` (e.g., `AppState`, `BalanceInfo`, `AppSettings`, `LogItem`)
- Functions: `snake_case` (e.g., `fetch_balance`, `spawn_refresh_loop`, `update_tray`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `STORE_FILE`)
- Tauri commands: `snake_case` matching the JS invoke string (e.g., `get_balance`, `save_settings`)

**JavaScript Identifiers:**
- Functions: `camelCase` (e.g., `loadBalance`, `updateUI`, `switchTab`, `applyTimeRange`)
- State objects: `camelCase` (e.g., `logState`)
- DOM element IDs: `kebab-case` (e.g., `pct-text`, `btn-refresh`, `filter-model`)
- CSS classes: `kebab-case` (e.g., `tab-btn`, `log-item`, `model-bar-row`)

## Where to Add New Code

**New Tauri command (backend feature):**
- Implementation: add `async fn` to `src-tauri/src/lib.rs` with `#[tauri::command]`
- Register: add to `invoke_handler!` macro in `run()` in `src-tauri/src/lib.rs`
- If HTTP: add fetch function to `src-tauri/src/api.rs`

**New frontend feature / tab:**
- Markup: add tab button and `<div class="tab-content">` section to `src/index.html`
- Logic: add functions and event listeners to `src/popup.js`
- Styles: add rules to `src/styles.css`
- Register tab size: add entry to `TAB_SIZES` object in `src/popup.js` (line 10)

**New settings field:**
- Add field to `AppSettings` struct in `src-tauri/src/lib.rs` with `#[serde(default)]`
- Add `default_*` function if non-trivial default
- Add `store.get` / `store.set` calls in `src-tauri/src/store.rs`
- Add form element to settings tab in `src/index.html`
- Add read/write in `loadSettings()` and save handler in `src/popup.js`

**New Rust module:**
- Create `src-tauri/src/{module_name}.rs`
- Declare with `mod {module_name};` in `src-tauri/src/lib.rs`

**Utilities:**
- Shared Rust helpers: add to `src-tauri/src/lib.rs` as private functions, or create a new module
- Shared JS helpers: add to `src/popup.js` (no module system — all in one file)

## Special Directories

**`.planning/`:**
- Purpose: GSD planning and codebase analysis documents
- Generated: No (hand-maintained by GSD commands)
- Committed: Yes

**`src-tauri/target/`:**
- Purpose: Rust compiler output and build cache
- Generated: Yes
- Committed: No

---

*Structure analysis: 2026-03-17*
