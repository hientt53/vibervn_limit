# External Integrations

**Analysis Date:** 2026-03-17

## APIs & External Services

**Viber Balance API (primary):**
- Service: `viber.claudegateway.site` — proprietary API gateway for balance/usage data
  - SDK/Client: `reqwest 0.12` (raw HTTP, no SDK) in `src-tauri/src/api.rs`
  - Auth: Bearer token supplied by user at runtime, stored via `tauri-plugin-store`
  - Endpoints used:
    - `GET https://viber.claudegateway.site/api/balance/check` — fetch current balance
    - `GET https://viber.claudegateway.site/api/balance/logs` — paginated usage logs
    - `GET https://viber.claudegateway.site/api/balance/logs/stat` — aggregated log stats
  - Custom headers: `New-API-User: -1`, `Cache-Control: no-store`
  - Timeout: 10s (balance), 15s (logs/stats)
  - Error handling: 401/403 → auth error string; non-2xx → HTTP status string

## Data Storage

**Databases:**
- None — no external database

**Local Persistence:**
- `tauri-plugin-store 2` — JSON key-value store
  - File: `settings.json` in Tauri app data directory
  - Implementation: `src-tauri/src/store.rs`
  - Keys stored: `token`, `refresh_minutes`, `show_percent_text`, `theme`, `alert_threshold`, `auto_start`

**File Storage:**
- Local filesystem only (CSV export written as string returned to frontend)

**Caching:**
- None — `Cache-Control: no-store` on all API requests; in-memory only via `AppState.last_balance` Mutex

## Authentication & Identity

**Auth Provider:**
- Custom — user-supplied Bearer token for `viber.claudegateway.site`
  - Implementation: token stored in `tauri-plugin-store`, injected as `Authorization: Bearer <token>` header in `src-tauri/src/api.rs`
  - No OAuth, no session management

## Monitoring & Observability

**Error Tracking:**
- None — errors logged to stderr via `eprintln!` in `src-tauri/src/lib.rs`

**Logs:**
- `eprintln!` for API errors in background refresh loop (`src-tauri/src/lib.rs` line ~306)
- Frontend errors emitted as Tauri events (`balance-error`)

## CI/CD & Deployment

**Hosting:**
- GitHub Releases — binaries uploaded as release artifacts

**CI Pipeline:**
- GitHub Actions — `.github/workflows/` (Build & Release workflow)
  - Triggers: push to `main`, version tags (`v*`), manual dispatch
  - Matrix: macOS arm64, macOS x86_64, Ubuntu 22.04, Windows
  - Uses: `actions/checkout@v4`, `actions/setup-node@v4` (LTS), `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`

## Environment Configuration

**Required env vars:**
- None at runtime — all configuration is user-supplied via the in-app settings UI

**Secrets location:**
- API token stored in Tauri's app data directory (`settings.json` via `tauri-plugin-store`)
- No `.env` files present or required

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None — app is read-only consumer of the balance API

## Desktop System Integrations

**System Tray:**
- Tauri tray icon (`tray-icon` feature) with dynamically generated battery icon
- macOS: title text shown in menu bar; Windows/Linux: tooltip on hover
- Implementation: `src-tauri/src/lib.rs` (`update_tray`, `TrayIconBuilder`)

**Notifications:**
- `tauri-plugin-notification 2` — native OS notification when balance drops below threshold
- Triggered in background refresh loop (`src-tauri/src/lib.rs`)

**Autostart:**
- `tauri-plugin-autostart 2` — macOS LaunchAgent, toggled via `toggle_autostart` Tauri command
- Implementation: `src-tauri/src/lib.rs`

**Window Vibrancy:**
- `window-vibrancy 0.7.1` — applies `NSVisualEffectMaterial::Popover` blur on macOS popup window
- Implementation: `src-tauri/src/lib.rs` (`open_or_focus_window`)

---

*Integration audit: 2026-03-17*
