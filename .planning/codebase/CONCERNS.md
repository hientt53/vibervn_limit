# Codebase Concerns

**Analysis Date:** 2026-03-17

## Tech Debt

**Hardcoded API base URL:**
- Issue: The API base URL `https://viber.claudegateway.site` is hardcoded in two separate places with no constant or config abstraction.
- Files: `src-tauri/src/api.rs` (lines 35, 163, 208)
- Impact: Changing the API endpoint requires editing multiple locations; no way to point at a staging/dev server without a code change.
- Fix approach: Extract to a `const BASE_URL: &str` at the top of `api.rs` and reference it throughout.

**Duplicate HTTP client construction:**
- Issue: `fetch_balance` builds its own `reqwest::Client` inline (line 28–32) while `fetch_logs` and `fetch_log_stats` use a shared `build_client()` helper. Two different timeout values are used (10s vs 15s) with no explanation.
- Files: `src-tauri/src/api.rs`
- Impact: Inconsistent timeout behavior; `fetch_balance` doesn't benefit from connection pooling that a shared client would provide.
- Fix approach: Consolidate into a single `build_client()` call used by all functions, with a single configurable timeout constant.

**Hardcoded quota-to-USD conversion factor:**
- Issue: The magic number `500000` (quota units per USD) appears in three places: `export_logs_csv` in `lib.rs` (line 159), and twice in `popup.js` (lines 164, 226, 411).
- Files: `src-tauri/src/lib.rs` (line 159), `src/popup.js` (lines 164, 226, 411)
- Impact: If the pricing changes, all occurrences must be updated manually; easy to miss one.
- Fix approach: Define as a named constant in Rust (`const QUOTA_PER_USD: f64 = 500_000.0`) and a JS constant (`const QUOTA_PER_USD = 500_000`).

**Dead/orphaned files:**
- Issue: `src/settings.html` and `src/settings.js` are fully implemented settings pages that are no longer used. The settings UI was consolidated into the main popup tab (`src/index.html`), but the old files were not removed.
- Files: `src/settings.html`, `src/settings.js`
- Impact: Confusing to future contributors; `settings.js` duplicates save/load logic already in `popup.js`; the `capabilities/default.json` still lists `"settings"` as a valid window target.
- Fix approach: Delete `src/settings.html` and `src/settings.js`; remove `"settings"` from the `windows` array in `src-tauri/capabilities/default.json`.

**`test_icon.rs` in project root:**
- Issue: A stray `test_icon.rs` file (63 bytes) sits in the project root, outside of `src-tauri/src/`.
- Files: `/test_icon.rs`
- Impact: Not compiled, not tested, misleading to contributors.
- Fix approach: Delete or move into `src-tauri/src/` with proper module wiring.

**`logsLoaded` / `settingsLoaded` flags never reset:**
- Issue: In `popup.js`, `logsLoaded` and `settingsLoaded` are module-level booleans that prevent re-loading on tab switch. If settings are saved and the user switches away and back, the settings form is not refreshed.
- Files: `src/popup.js` (lines 304–323)
- Impact: Stale settings displayed after a save-and-switch workflow.
- Fix approach: Reset `settingsLoaded = false` after a successful save, or always reload on tab focus.

**`shortModel` uses hardcoded date suffixes:**
- Issue: `shortModel()` strips specific date strings (`-20251001`, `-20250514`) from model names.
- Files: `src/popup.js` (lines 184–189)
- Impact: New model versions with different date suffixes will display with the raw suffix appended, requiring a code update each time Anthropic releases a new model.
- Fix approach: Use a regex like `/-\d{8}$/` to strip any 8-digit date suffix generically.

## Security Considerations

**CSP disabled:**
- Risk: `tauri.conf.json` sets `"csp": null`, disabling Content Security Policy entirely for the webview.
- Files: `src-tauri/tauri.conf.json` (line 26)
- Current mitigation: The frontend loads no external scripts; all JS is local.
- Recommendations: Define a restrictive CSP (e.g., `"default-src 'self'"`) to prevent any future accidental inline script injection or remote resource loading.

**API token stored in plaintext:**
- Risk: The Bearer token is stored via `tauri-plugin-store` in a plain JSON file (`settings.json`) in the app data directory. No encryption at rest.
- Files: `src-tauri/src/store.rs`
- Current mitigation: File is in the OS user data directory, not world-readable by default.
- Recommendations: Use the OS keychain (via `tauri-plugin-stronghold` or `keyring`) for token storage rather than a plain JSON file.

**Token transmitted in Authorization header over HTTPS only:**
- Risk: If the hardcoded API URL were ever changed to HTTP (e.g., during development), the token would be sent in plaintext.
- Files: `src-tauri/src/api.rs`
- Current mitigation: URL is currently HTTPS.
- Recommendations: Add a runtime assertion or `reqwest` builder option to enforce HTTPS-only connections.

**`macOSPrivateApi: true` enabled:**
- Risk: Enables private macOS APIs which may cause App Store rejection or unexpected behavior on future macOS versions.
- Files: `src-tauri/tauri.conf.json` (line 24), `src-tauri/Cargo.toml` (line 16)
- Current mitigation: Used for `window-vibrancy` blur effect only.
- Recommendations: Document the specific reason this is needed; consider whether the vibrancy effect is worth the risk.

## Performance Bottlenecks

**Per-request HTTP client construction in `fetch_balance`:**
- Problem: A new `reqwest::Client` is built on every balance poll cycle (every N minutes).
- Files: `src-tauri/src/api.rs` (lines 28–32)
- Cause: Client is not shared or cached; connection pool is discarded after each call.
- Improvement path: Create the client once at app startup, store it in `AppState`, and pass it to API functions.

**`export_logs_csv` fetches up to 1000 items in one request:**
- Problem: The CSV export hardcodes `page_size=1000` with no pagination or streaming.
- Files: `src-tauri/src/lib.rs` (lines 150–163)
- Cause: Simple single-request implementation.
- Improvement path: For large datasets, paginate and stream CSV rows, or warn the user if total exceeds a threshold.

**Icon rendered per-pixel with 4x supersampling on every tray update:**
- Problem: `generate_battery_icon` iterates every pixel with a 4x4 subpixel grid (16 samples/pixel) on the CPU for every tray refresh.
- Files: `src-tauri/src/icon.rs`
- Cause: No caching; icon is regenerated even if the percentage hasn't changed.
- Improvement path: Cache the last rendered percentage and skip regeneration if the value hasn't changed by more than 1%.

## Fragile Areas

**Refresh loop interval reset logic:**
- Files: `src-tauri/src/lib.rs` (lines 256–272)
- Why fragile: When `refresh_minutes` changes, the loop resets the `tokio::time::interval` and immediately calls `ticker.tick().await` to consume the instant tick. If settings change rapidly, multiple interval resets can stack up in unexpected ways.
- Safe modification: Add a dedicated channel (e.g., `tokio::sync::watch`) to signal interval changes rather than polling inside the loop.
- Test coverage: No tests exist for the refresh loop behavior.

**`waitForTauri` polling loop has no timeout:**
- Files: `src/popup.js` (lines 1–8), `src/settings.js` (lines 1–8)
- Why fragile: If `window.__TAURI__` never becomes available (e.g., webview injection failure), the interval runs forever with no error surfaced to the user.
- Safe modification: Add a timeout (e.g., 5 seconds) after which the promise rejects with a descriptive error.

**`open_or_focus_window` silently ignores build errors:**
- Files: `src-tauri/src/lib.rs` (lines 228–250)
- Why fragile: The `win` result from `WebviewWindowBuilder::build()` is matched with `if let Ok(w)`, silently discarding any window creation error. The vibrancy call also uses `.ok()` to swallow errors.
- Safe modification: Log errors from window creation; surface them to the user if the window fails to open.

**`logState.knownModels` grows unbounded:**
- Files: `src/popup.js` (lines 106–115, 135–138)
- Why fragile: Model names are added to a `Set` that is never cleared. If the user browses many pages across different time ranges, the model filter dropdown accumulates all ever-seen models for the session.
- Safe modification: Rebuild `knownModels` from the current page's results rather than accumulating across pages.

## Scaling Limits

**Log export hard cap at 1000 items:**
- Current capacity: 1000 log entries per export
- Limit: Users with high API usage will silently get a truncated export with no warning.
- Scaling path: Implement paginated export or display a count warning before export.

**Single-threaded Mutex state:**
- Current capacity: All state access serializes through `std::sync::Mutex`. Adequate for current load (one background loop + occasional UI commands).
- Limit: If more concurrent background tasks are added, lock contention could cause UI commands to block.
- Scaling path: Use `tokio::sync::RwMutex` for read-heavy state like `last_balance` and `settings`.

## Dependencies at Risk

**`window-vibrancy = "0.7.1"` (macOS-only):**
- Risk: This crate uses private macOS APIs. It may break on future macOS versions without notice.
- Impact: Vibrancy effect on the popup window would fail silently (already wrapped in `.ok()`).
- Migration plan: Monitor upstream for updates; the `.ok()` fallback means the app continues to function without the effect.

**`image = "0.25"` dependency unused in final binary:**
- Risk: The `image` crate is listed in `Cargo.toml` but the icon generation in `icon.rs` does all pixel manipulation manually without using it.
- Impact: Unnecessary compile-time dependency adding to binary size and build time.
- Migration plan: Remove `image` from `Cargo.toml` if it is confirmed unused.

## Test Coverage Gaps

**No tests exist anywhere:**
- What's not tested: The entire codebase — balance parsing logic, icon generation, store serialization, API response handling, quota conversion math, and all frontend logic.
- Files: `src-tauri/src/api.rs`, `src-tauri/src/icon.rs`, `src-tauri/src/store.rs`, `src-tauri/src/lib.rs`, `src/popup.js`
- Risk: Regressions in balance calculation, cost display, or API parsing go undetected until a user reports them.
- Priority: High — `parse_balance` in `api.rs` and the quota-to-USD conversion are the most critical to unit test given they directly affect displayed financial data.

---

*Concerns audit: 2026-03-17*
