# Technology Stack

**Analysis Date:** 2026-03-17

## Languages

**Primary:**
- Rust (edition 2021) - Backend/core logic in `src-tauri/src/`
- JavaScript (ES modules) - Frontend UI in `src/popup.js`, `src/settings.js`
- HTML/CSS - UI markup and styling in `src/index.html`, `src/styles.css`, `src/settings.html`

## Runtime

**Environment:**
- Tauri 2 desktop runtime (wraps a WebView for frontend, Rust for backend)
- Node.js LTS (dev tooling only — Tauri CLI)

**Package Manager:**
- npm (Node) — lockfileVersion 3, `package-lock.json` present
- Cargo (Rust) — `src-tauri/Cargo.lock` present

## Frameworks

**Core:**
- Tauri 2 (`tauri = "2"`, features: `macos-private-api`, `tray-icon`) — desktop app shell, tray icon, IPC bridge
- tokio 1 (features: `full`) — async runtime for Rust backend

**Build/Dev:**
- `@tauri-apps/cli ^2` (npm devDependency) — Tauri build toolchain
- `tauri-build = "2"` — Rust build script (`src-tauri/build.rs`)

**Testing:**
- Not detected

## Key Dependencies

**Critical:**
- `reqwest 0.12` (features: `json`, `gzip`) — HTTP client for API calls in `src-tauri/src/api.rs`
- `serde 1` + `serde_json 1` — JSON serialization/deserialization throughout `src-tauri/src/`
- `tauri-plugin-store 2` — persistent key-value settings storage (`src-tauri/src/store.rs`)
- `tauri-plugin-autostart 2` — launch-at-login support (`src-tauri/src/lib.rs`)
- `tauri-plugin-notification 2` — low-balance desktop notifications (`src-tauri/src/lib.rs`)

**Infrastructure:**
- `window-vibrancy 0.7.1` — macOS NSVisualEffectMaterial blur on popup window (`src-tauri/src/lib.rs`)
- `image 0.25` — programmatic RGBA battery icon generation (`src-tauri/src/icon.rs`)
- `chrono 0.4` — timestamp arithmetic for daily stats (`src-tauri/src/lib.rs`)
- `tauri-plugin-shell 2` — shell access plugin
- `tauri-plugin-dialog 2` — native dialog plugin

## Configuration

**Environment:**
- No `.env` files — no environment variables required at runtime
- API token stored at runtime via `tauri-plugin-store` in `settings.json` (app data dir)
- All settings persisted locally: token, refresh interval, theme, alert threshold, auto-start

**Build:**
- `src-tauri/tauri.conf.json` — Tauri app config (product name, window config, bundle targets, CSP)
- `src-tauri/Cargo.toml` — Rust dependencies and crate metadata
- `package.json` — npm scripts (`tauri` command only)

## Platform Requirements

**Development:**
- Node.js LTS (for `@tauri-apps/cli`)
- Rust stable toolchain
- macOS: Xcode command line tools; Linux: GTK/WebKit2GTK system libs

**Production:**
- Targets: macOS (arm64 + x86_64), Linux, Windows (all via GitHub Actions matrix)
- macOS: runs as menu bar accessory app (no Dock icon), uses `macOSPrivateApi`
- Bundle format: determined by `"targets": "all"` in `src-tauri/tauri.conf.json`

---

*Stack analysis: 2026-03-17*
