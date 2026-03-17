# Coding Conventions

**Analysis Date:** 2026-03-17

## Naming Patterns

**Files (Rust):**
- `snake_case.rs` — all Rust source files: `api.rs`, `icon.rs`, `store.rs`, `lib.rs`, `main.rs`

**Files (Frontend):**
- `kebab-case` for HTML/CSS: `index.html`, `styles.css`
- `camelCase.js` for JavaScript: `popup.js`, `settings.js`

**Functions (Rust):**
- `snake_case` for all functions: `fetch_balance`, `parse_balance`, `build_client`, `spawn_refresh_loop`, `format_label`, `update_tray`, `tray_icon_size`
- Tauri commands follow the same `snake_case`: `get_balance`, `save_settings`, `refresh_now`, `export_logs_csv`

**Functions (JavaScript):**
- `camelCase` for all functions: `loadBalance`, `updateUI`, `renderLogs`, `loadLogs`, `switchTab`, `applyTimeRange`, `loadSettings`, `waitForTauri`

**Variables (Rust):**
- `snake_case` for all variables and struct fields: `used_quota_usd`, `remain_quota_usd`, `next_reset_time`, `refresh_minutes`

**Variables (JavaScript):**
- `camelCase` for local variables: `logState`, `totalQuota`, `totalTokens`, `logsLoaded`
- `SCREAMING_SNAKE_CASE` for module-level constants: `TAB_SIZES`, `MODEL_COLORS`

**Types/Structs (Rust):**
- `PascalCase` for all types: `BalanceInfo`, `AppSettings`, `AppState`, `LogItem`, `LogsPage`, `LogStatItem`
- Private API response types also `PascalCase`: `ApiResponse`, `ApiData`, `TokenInfo`, `LogsApiResponse`

**CSS Classes:**
- `kebab-case` for all class names: `.popup-container`, `.tab-btn`, `.log-item`, `.stat-box`, `.model-bar-row`
- BEM-like naming for sub-elements: `.log-item-top`, `.log-item-bottom`, `.model-bar-fill`, `.model-bar-track`

**HTML IDs:**
- `kebab-case`: `#pct-text`, `#progress-ring-fill`, `#stat-remaining`, `#btn-refresh`, `#filter-model`

## Code Style

**Formatting (Rust):**
- Standard `rustfmt` conventions (no explicit config file present)
- Trailing commas in multi-line struct literals and function calls
- Inline single-expression functions on one line: `fn default_theme() -> String { "system".to_string() }`
- Block comments use `// ── Section Name ──` separator style for visual grouping

**Formatting (JavaScript):**
- 2-space indentation
- Single quotes for strings
- Arrow functions for callbacks
- Template literals for string interpolation
- No semicolons omitted (semicolons used consistently)

**Formatting (CSS):**
- 2-space indentation
- Section comments use `/* ── SECTION ─────── */` style
- CSS custom properties (variables) defined in `:root`

## Import Organization

**Rust — Order:**
1. External crates (`tauri`, `serde`, `std`)
2. Local modules via `mod` declarations
3. `use crate::` for internal types

Example from `src-tauri/src/lib.rs`:
```rust
use tauri::{...};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

mod api;
mod icon;
mod store;
```

**JavaScript:**
- No module imports — all code uses `window.__TAURI__` global namespace
- Tauri APIs accessed via: `window.__TAURI__.core.invoke(...)`, `window.__TAURI__.event.listen(...)`, `window.__TAURI__.webviewWindow`, `window.__TAURI__.dpi`

## Error Handling

**Rust — Tauri Commands:**
- All commands return `Result<T, String>` — errors are `.to_string()` conversions
- Use `.map_err(|e| e.to_string())` for converting library errors
- Use `.map_err(|e| format!("Context: {e}"))` when adding context
- Use `?` operator for propagation within commands
- Early returns with `Err("message".to_string())` for validation

Example from `src-tauri/src/lib.rs`:
```rust
async fn save_settings(...) -> Result<(), String> {
    store::save(&app, &new_settings).map_err(|e| e.to_string())?;
    ...
}
```

**Rust — Non-command functions:**
- Fire-and-forget with `.ok()` for non-critical operations: `app.emit("event", ()).ok()`
- `eprintln!` for logging errors in background loops: `eprintln!("API error: {e}")`
- `unwrap_or_default()` and `unwrap_or(value)` for store reads with fallbacks

**JavaScript:**
- `try/catch` wrapping all `invoke()` calls
- `console.error('context:', e)` for logging
- UI feedback on error: update element text with `Error: ${e}`
- Silent catch for non-critical operations: `.catch(() => {})`

## Logging

**Rust:**
- `eprintln!` for error output in background tasks (no structured logging library)
- No info/debug logging — errors only

**JavaScript:**
- `console.error('context message:', e)` pattern throughout
- No `console.log` or `console.warn` usage

## Comments

**Rust:**
- Doc comments (`///`) used for public functions: `/// Returns the platform-appropriate tray icon dimensions.`
- Inline comments for non-obvious logic: `// consume immediate tick from reset`
- Section separators: `// --- Section Name ---` and `// ── Section Name ──`
- `#[cfg(...)]` blocks always have a comment explaining the platform behavior

**JavaScript:**
- Section separators: `// ── Section Name ──────────────────────`
- Inline comments for non-obvious logic
- No JSDoc used

**HTML:**
- Section comments: `<!-- ═══ Section Name ═══ -->`

## Function Design

**Rust:**
- Small, focused functions — `parse_balance`, `format_label`, `tray_icon_size` are pure helpers
- Async functions for all I/O: `fetch_balance`, `fetch_logs`, `fetch_log_stats`
- Sync functions for pure computation and state mutation: `format_label`, `update_tray`
- Background work spawned via `tauri::async_runtime::spawn`

**JavaScript:**
- `async/await` for all Tauri invocations
- DOM manipulation functions are synchronous: `renderLogs`, `updatePagination`, `updateModelFilter`
- Lazy loading pattern: `logsLoaded` / `settingsLoaded` flags prevent redundant fetches

## Module Design

**Rust:**
- Three focused modules: `api` (HTTP), `icon` (rendering), `store` (persistence)
- `lib.rs` is the orchestration layer — holds state types, Tauri commands, and app setup
- `main.rs` is a thin entry point: just calls `tauri_app_lib::run()`
- Internal API types (response shapes) are private (`struct ApiResponse`, not `pub`)
- Public types are those shared with the frontend via Tauri commands

**JavaScript:**
- Single file `popup.js` — no module system
- State managed via module-level `let` variables: `logState`, `logsLoaded`, `settingsLoaded`
- `window._nextResetTime` used for cross-function state (global window property)

## Platform-Specific Code

**Rust:**
- `#[cfg(target_os = "macos")]` / `#[cfg(not(target_os = "macos"))]` for platform branches
- `#[cfg(not(debug_assertions))]` in `main.rs` for Windows console suppression
- Platform blocks kept inline, not extracted to separate files

---

*Convention analysis: 2026-03-17*
