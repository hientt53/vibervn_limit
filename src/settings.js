const { invoke } = window.__TAURI__.core;

async function load() {
  try {
    const s = await invoke('get_settings');
    document.getElementById('token-input').value = s.token || '';
    document.getElementById('interval-input').value = s.refresh_minutes || 5;
    document.getElementById('show-text-check').checked = s.show_percent_text !== false;
    document.getElementById('theme-select').value = s.theme || 'system';
    document.getElementById('alert-threshold').value = s.alert_threshold ?? 20;
    document.getElementById('auto-start-check').checked = s.auto_start === true;

    // Apply theme immediately
    applyTheme(s.theme || 'system');
  } catch (e) {
    console.error('loadSettings error:', e);
  }
}

function applyTheme(theme) {
  document.documentElement.classList.remove('theme-light', 'theme-dark');
  if (theme === 'light') document.documentElement.classList.add('theme-light');
  else if (theme === 'dark') document.documentElement.classList.add('theme-dark');
}

document.getElementById('theme-select').addEventListener('change', (e) => {
  applyTheme(e.target.value);
});

document.getElementById('toggle-token').addEventListener('click', () => {
  const inp = document.getElementById('token-input');
  inp.type = inp.type === 'password' ? 'text' : 'password';
});

// Clamp interval value on blur
document.getElementById('interval-input').addEventListener('blur', (e) => {
  let val = parseInt(e.target.value, 10);
  if (isNaN(val) || val < 1) val = 1;
  if (val > 60) val = 60;
  e.target.value = val;
});

// Clamp threshold on blur
document.getElementById('alert-threshold').addEventListener('blur', (e) => {
  let val = parseInt(e.target.value, 10);
  if (isNaN(val) || val < 0) val = 0;
  if (val > 100) val = 100;
  e.target.value = val;
});

document.getElementById('btn-save').addEventListener('click', async () => {
  const settings = {
    token: document.getElementById('token-input').value.trim(),
    refresh_minutes: parseInt(document.getElementById('interval-input').value, 10) || 5,
    show_percent_text: document.getElementById('show-text-check').checked,
    theme: document.getElementById('theme-select').value,
    alert_threshold: parseInt(document.getElementById('alert-threshold').value, 10) || 20,
    auto_start: document.getElementById('auto-start-check').checked,
  };
  try {
    await invoke('save_settings', { newSettings: settings });
    // Sync OS auto-start with setting
    await invoke('toggle_autostart', { enabled: settings.auto_start }).catch(() => {});
    const status = document.getElementById('save-status');
    status.textContent = '✓ Saved! Refreshing balance…';
    status.classList.add('success');
    setTimeout(() => {
      status.textContent = '';
      status.classList.remove('success');
    }, 2500);
  } catch (e) {
    document.getElementById('save-status').textContent = `Error: ${e}`;
  }
});

document.getElementById('btn-cancel').addEventListener('click', async () => {
  const { getCurrentWebviewWindow } = window.__TAURI__.webviewWindow;
  getCurrentWebviewWindow().close();
});

// ── Keyboard Shortcuts ────────────────────

document.addEventListener('keydown', (e) => {
  const mod = e.metaKey || e.ctrlKey;
  if (mod && e.key === 'w') {
    e.preventDefault();
    const { getCurrentWebviewWindow } = window.__TAURI__.webviewWindow;
    getCurrentWebviewWindow().close();
  } else if (e.key === 'Escape') {
    e.preventDefault();
    const { getCurrentWebviewWindow } = window.__TAURI__.webviewWindow;
    getCurrentWebviewWindow().close();
  }
});

window.addEventListener('DOMContentLoaded', load);
