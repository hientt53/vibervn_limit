# Viber Balance

A macOS menu bar app built with Tauri + Rust that monitors your [Viber VN](https://viber.vn) API balance in real time.

## Features

- Menu bar tray icon with a live battery-style indicator showing remaining balance %
- Popup window with balance stats (remaining, used, total) and reset countdown
- Usage logs with filtering by model, time range, and pagination
- Model cost breakdown and 7-day usage chart
- Export logs to CSV
- Low balance notification alert (configurable threshold)
- Light / Dark / System theme
- Launch at login (auto-start)
- Configurable refresh interval (1–60 minutes)
- Keyboard shortcuts: `⌘R` refresh, `⌘,` settings, `⌘W` close

## Requirements

- macOS (primary target)
- [Rust](https://rustup.rs/) toolchain
- [Node.js](https://nodejs.org/) + npm
- [Tauri CLI v2](https://tauri.app/start/prerequisites/)

## Getting Started

```bash
# Install JS dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Configuration

On first launch, click the tray icon → Settings (or press `⌘,`) and enter your Viber VN API token.

| Setting | Default | Description |
|---|---|---|
| Token | — | Your Viber VN API token |
| Refresh interval | 5 min | How often to poll the API |
| Alert threshold | 20% | Notify when balance drops below this |
| Show % text | on | Display percentage next to tray icon |
| Theme | system | `light`, `dark`, or `system` |
| Launch at login | off | Auto-start on macOS login |

## Project Structure

```
src/              # Frontend (HTML + vanilla JS)
  index.html      # Popup window
  popup.js        # Balance view, logs, charts
  settings.html   # Settings window
  settings.js     # Settings form logic
  styles.css      # Shared styles

src-tauri/        # Rust backend (Tauri)
  src/
    lib.rs        # App setup, tray, commands, refresh loop
    api.rs        # Viber VN API client
    store.rs      # Persistent settings (tauri-plugin-store)
    icon.rs       # Dynamic battery tray icon generator
```

## Tech Stack

- [Tauri v2](https://tauri.app/) — native shell
- Rust — backend, API calls, tray icon rendering
- Vanilla JS — frontend UI (no framework)
- `tauri-plugin-store` — settings persistence
- `tauri-plugin-notification` — low balance alerts
- `tauri-plugin-autostart` — launch at login
- `window-vibrancy` — macOS blur effect

