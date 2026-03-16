function waitForTauri() {
  return new Promise((resolve) => {
    if (window.__TAURI__) { resolve(); return; }
    const id = setInterval(() => {
      if (window.__TAURI__) { clearInterval(id); resolve(); }
    }, 50);
  });
}

const TAB_SIZES = {
  balance:  { w: 320, h: 310 },
  logs:     { w: 380, h: 560 },
  settings: { w: 380, h: 480 },
};

async function resizeForTab(tab) {
  const size = TAB_SIZES[tab] || TAB_SIZES.balance;
  try {
    const { getCurrentWebviewWindow } = window.__TAURI__.webviewWindow;
    const { LogicalSize } = window.__TAURI__.dpi;
    const win = getCurrentWebviewWindow();
    await win.setSize(new LogicalSize(size.w, size.h));
  } catch (e) {
    console.error('resize error:', e);
  }
}

// ── Balance View ──────────────────────────

async function loadBalance() {
  try {
    const balance = await window.__TAURI__.core.invoke('get_balance');
    if (balance) updateUI(balance);
  } catch (e) {
    console.error('loadBalance error:', e);
  }
}

function updateUI(balance) {
  const pct = balance.unlimited ? 100 : balance.percent;
  document.getElementById('pct-text').textContent = balance.unlimited ? '∞' : `${Math.round(pct)}%`;

  const fill = document.getElementById('battery-fill');
  fill.style.width = `${pct}%`;
  fill.className = 'battery-fill';
  if (!balance.unlimited) {
    if (pct <= 20) fill.classList.add('red');
    else if (pct <= 50) fill.classList.add('yellow');
  }

  const fmt = (n) => `$${n.toFixed(2)}`;
  document.getElementById('stat-remaining').textContent = fmt(balance.remain_usd);
  document.getElementById('stat-used').textContent = fmt(balance.used_usd);
  document.getElementById('stat-total').textContent = fmt(balance.total_usd);

  // Reset countdown
  const resetRow = document.getElementById('reset-row');
  if (balance.next_reset_time) {
    resetRow.style.display = 'flex';
    window._nextResetTime = balance.next_reset_time;
    updateResetCountdown();
  } else {
    resetRow.style.display = 'none';
  }

  document.getElementById('last-update').textContent = `Updated ${new Date().toLocaleTimeString()}`;
}

function updateResetCountdown() {
  if (!window._nextResetTime) return;
  const now = Math.floor(Date.now() / 1000);
  const diff = window._nextResetTime - now;
  const el = document.getElementById('reset-text');

  if (diff <= 0) {
    el.textContent = 'Reset now!';
    el.style.color = 'var(--sys-green)';
    return;
  }

  const days = Math.floor(diff / 86400);
  const hours = Math.floor((diff % 86400) / 3600);
  const mins = Math.floor((diff % 3600) / 60);

  let parts = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  parts.push(`${mins}m`);

  const resetDate = new Date(window._nextResetTime * 1000);
  const dateStr = resetDate.toLocaleDateString([], { month: 'short', day: 'numeric' });
  const timeStr = resetDate.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

  el.textContent = `Reset in ${parts.join(' ')} — ${dateStr} ${timeStr}`;
}

setInterval(updateResetCountdown, 60000);

// ── Logs View ─────────────────────────────

let logState = {
  page: 1,
  pageSize: 15,
  total: 0,
  modelName: '',
  timeRange: 'today',
  startTimestamp: null,
  endTimestamp: null,
  knownModels: new Set(),
};

async function loadLogs(page = 1) {
  logState.page = page;
  const list = document.getElementById('log-list');
  list.innerHTML = '<div class="log-empty">Loading…</div>';

  try {
    const result = await window.__TAURI__.core.invoke('get_logs', {
      page: logState.page,
      pageSize: logState.pageSize,
      logType: 0,
      modelName: logState.modelName || null,
      startTimestamp: logState.startTimestamp,
      endTimestamp: logState.endTimestamp,
    });

    logState.total = result.total;

    // Collect model names for filter dropdown
    result.items.forEach((item) => {
      if (item.model_name) logState.knownModels.add(item.model_name);
    });
    updateModelFilter();

    renderLogs(result.items);
    updatePagination();
    loadLogStats();
  } catch (e) {
    console.error('loadLogs error:', e);
    list.innerHTML = `<div class="log-empty">Error: ${e}</div>`;
  }
}

function renderLogs(items) {
  const list = document.getElementById('log-list');
  if (!items.length) {
    list.innerHTML = '<div class="log-empty">No logs found</div>';
    return;
  }

  list.innerHTML = items
    .map((item) => {
      const time = new Date(item.created_at * 1000);
      const timeStr = time.toLocaleString([], {
        month: 'short', day: 'numeric',
        hour: '2-digit', minute: '2-digit',
      });
      const model = shortModel(item.model_name);
      const cost = `$${(item.quota / 500000).toFixed(4)}`;
      const tokens = item.prompt_tokens + item.completion_tokens;
      const hasError = item.content && item.content.length > 0;
      const streamIcon = item.is_stream ? '⇄' : '→';

      return `<div class="log-item${hasError ? ' log-error' : ''}">
        <div class="log-item-top">
          <span class="log-model">${model}</span>
          <span class="log-cost">${cost}</span>
        </div>
        <div class="log-item-bottom">
          <span class="log-tokens">${streamIcon} ${fmtNum(tokens)} tok · ${item.use_time}s</span>
          <span class="log-time">${timeStr}</span>
        </div>
        ${hasError ? `<div class="log-content">${escHtml(item.content)}</div>` : ''}
      </div>`;
    })
    .join('');
}

function shortModel(name) {
  return name
    .replace('claude-', '')
    .replace('-20251001', '')
    .replace('-20250514', '');
}

function fmtNum(n) {
  if (n >= 1000000) return (n / 1000000).toFixed(1) + 'M';
  if (n >= 1000) return (n / 1000).toFixed(1) + 'k';
  return String(n);
}

function escHtml(s) {
  const d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}

async function loadLogStats() {
  try {
    const data = await window.__TAURI__.core.invoke('get_log_stats', {
      logType: 0,
      modelName: logState.modelName || null,
      startTimestamp: logState.startTimestamp,
      endTimestamp: logState.endTimestamp,
    });

    const statsEl = document.getElementById('log-stats');
    if (data && typeof data === 'object') {
      let totalQuota = 0, totalTokens = 0, totalCount = 0;

      // data can be an array or an object
      const items = Array.isArray(data) ? data : (data.items || [data]);
      items.forEach((s) => {
        totalQuota += s.quota || 0;
        totalTokens += (s.prompt_tokens || 0) + (s.completion_tokens || 0);
        totalCount += s.count || 0;
      });

      document.getElementById('stat-requests').textContent = fmtNum(totalCount);
      document.getElementById('stat-tokens').textContent = fmtNum(totalTokens);
      document.getElementById('stat-cost').textContent = `$${(totalQuota / 500000).toFixed(2)}`;
      statsEl.style.display = 'flex';
    } else {
      statsEl.style.display = 'none';
    }
  } catch (e) {
    console.error('loadLogStats error:', e);
    document.getElementById('log-stats').style.display = 'none';
  }
}

function updateModelFilter() {
  const sel = document.getElementById('filter-model');
  const current = sel.value;
  const models = [...logState.knownModels].sort();

  // Keep existing options, add missing ones
  const existing = new Set([...sel.options].map((o) => o.value));
  models.forEach((m) => {
    if (!existing.has(m)) {
      const opt = document.createElement('option');
      opt.value = m;
      opt.textContent = shortModel(m);
      sel.appendChild(opt);
    }
  });
  sel.value = current;
}

function updatePagination() {
  const totalPages = Math.max(1, Math.ceil(logState.total / logState.pageSize));
  document.getElementById('page-info').textContent = `${logState.page} / ${totalPages}`;
  document.getElementById('btn-prev').disabled = logState.page <= 1;
  document.getElementById('btn-next').disabled = logState.page >= totalPages;
}

function applyTimeRange(preset) {
  logState.timeRange = preset;
  const now = Math.floor(Date.now() / 1000);

  if (!preset) {
    logState.startTimestamp = null;
    logState.endTimestamp = null;
    return;
  }

  logState.endTimestamp = null; // always "until now"

  if (preset === '15m') {
    logState.startTimestamp = now - 15 * 60;
  } else if (preset === '30m') {
    logState.startTimestamp = now - 30 * 60;
  } else if (preset === '1h') {
    logState.startTimestamp = now - 3600;
  } else if (preset === '3h') {
    logState.startTimestamp = now - 3 * 3600;
  } else if (preset === '6h') {
    logState.startTimestamp = now - 6 * 3600;
  } else if (preset === '24h') {
    logState.startTimestamp = now - 24 * 3600;
  } else if (preset === 'today') {
    const d = new Date();
    d.setHours(0, 0, 0, 0);
    logState.startTimestamp = Math.floor(d.getTime() / 1000);
  } else if (preset === '7d') {
    logState.startTimestamp = now - 7 * 86400;
  } else if (preset === '30d') {
    logState.startTimestamp = now - 30 * 86400;
  } else if (preset === 'month') {
    const d = new Date();
    d.setDate(1);
    d.setHours(0, 0, 0, 0);
    logState.startTimestamp = Math.floor(d.getTime() / 1000);
  }
}

// ── Tab Switching ─────────────────────────

let logsLoaded = false;
let settingsLoaded = false;

function switchTab(tab) {
  document.querySelectorAll('.tab-btn').forEach((b) => {
    b.classList.toggle('active', b.dataset.tab === tab);
  });
  document.querySelectorAll('.tab-content').forEach((el) => el.classList.remove('active'));
  document.getElementById(`view-${tab}`).classList.add('active');
  resizeForTab(tab);

  if (tab === 'logs' && !logsLoaded) {
    logsLoaded = true;
    applyTimeRange(logState.timeRange);
    loadLogs(1);
  }
  if (tab === 'settings' && !settingsLoaded) {
    settingsLoaded = true;
    loadSettings();
  }
}

document.querySelectorAll('.tab-btn').forEach((btn) => {
  btn.addEventListener('click', () => switchTab(btn.dataset.tab));
});

// ── Event Listeners ───────────────────────

document.getElementById('btn-refresh').addEventListener('click', async () => {
  document.getElementById('last-update').textContent = 'Refreshing…';
  await window.__TAURI__.core.invoke('refresh_now');
});

document.getElementById('btn-settings').addEventListener('click', () => {
  switchTab('settings');
});


// ── Export CSV ─────────────────────────────

document.getElementById('btn-export').addEventListener('click', async () => {
  const btn = document.getElementById('btn-export');
  btn.disabled = true;
  btn.textContent = 'Exporting…';
  try {
    const csv = await window.__TAURI__.core.invoke('export_logs_csv', {
      logType: 0,
      modelName: logState.modelName || null,
      startTimestamp: logState.startTimestamp,
      endTimestamp: logState.endTimestamp,
    });
    // Trigger download via blob URL
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `viber-logs-${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  } catch (e) {
    console.error('export error:', e);
  } finally {
    btn.disabled = false;
    btn.innerHTML = `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg> CSV`;
  }
});

// Log filters
document.getElementById('filter-model').addEventListener('change', (e) => {
  logState.modelName = e.target.value;
  loadLogs(1);
});

document.getElementById('filter-time').addEventListener('change', (e) => {
  applyTimeRange(e.target.value);
  loadLogs(1);
});

// Pagination
document.getElementById('btn-prev').addEventListener('click', () => {
  if (logState.page > 1) loadLogs(logState.page - 1);
});

document.getElementById('btn-next').addEventListener('click', () => {
  const totalPages = Math.ceil(logState.total / logState.pageSize);
  if (logState.page < totalPages) loadLogs(logState.page + 1);
});

// ── Model Breakdown ───────────────────────

const MODEL_COLORS = [
  '#6366f1', '#8b5cf6', '#ec4899', '#f43f5e',
  '#f97316', '#eab308', '#22c55e', '#14b8a6',
  '#06b6d4', '#3b82f6',
];

async function loadModelBreakdown() {
  const section = document.getElementById('model-breakdown');
  if (!section) return;
  try {
    const data = await window.__TAURI__.core.invoke('get_log_stats', {
      logType: 0,
      modelName: null,
      startTimestamp: logState.startTimestamp,
      endTimestamp: logState.endTimestamp,
    });
    const items = Array.isArray(data) ? data : (data.items || [data]);
    // Aggregate by model
    const byModel = {};
    items.forEach((s) => {
      const name = s.model_name || 'other';
      if (!byModel[name]) byModel[name] = 0;
      byModel[name] += (s.quota || 0) / 500000;
    });
    const sorted = Object.entries(byModel).sort((a, b) => b[1] - a[1]);
    const total = sorted.reduce((sum, [, v]) => sum + v, 0);

    if (sorted.length === 0 || total === 0) {
      section.style.display = 'none';
      return;
    }
    section.style.display = 'block';
    const barsEl = section.querySelector('.model-bars') || (() => {
      const d = document.createElement('div');
      d.className = 'model-bars';
      section.appendChild(d);
      return d;
    })();
    barsEl.innerHTML = sorted.map(([model, cost], i) => {
      const pct = (cost / total * 100).toFixed(1);
      const color = MODEL_COLORS[i % MODEL_COLORS.length];
      return `<div class="model-bar-row">
        <span class="model-bar-label">${shortModel(model)}</span>
        <div class="model-bar-track">
          <div class="model-bar-fill" style="width:${pct}%;background:${color}"></div>
        </div>
        <span class="model-bar-value">$${cost.toFixed(2)}</span>
      </div>`;
    }).join('');
  } catch (e) {
    console.error('model breakdown error:', e);
    if (section) section.style.display = 'none';
  }
}

// ── Usage Chart ───────────────────────────

async function loadUsageChart() {
  const section = document.getElementById('usage-chart');
  if (!section) return;
  try {
    const data = await window.__TAURI__.core.invoke('get_log_stats', {
      logType: 0,
      modelName: null,
      startTimestamp: Math.floor(Date.now() / 1000) - 7 * 86400,
      endTimestamp: null,
    });
    const items = Array.isArray(data) ? data : (data.items || [data]);

    // Aggregate by day
    const byDay = {};
    items.forEach((s) => {
      if (!s.created_at) return;
      const d = new Date(s.created_at * 1000).toLocaleDateString([], { month: 'short', day: 'numeric' });
      if (!byDay[d]) byDay[d] = 0;
      byDay[d] += (s.quota || 0) / 500000;
    });

    // Fill last 7 days
    const days = [];
    for (let i = 6; i >= 0; i--) {
      const d = new Date(Date.now() - i * 86400 * 1000);
      const label = d.toLocaleDateString([], { month: 'short', day: 'numeric' });
      days.push({ label, cost: byDay[label] || 0 });
    }

    const maxCost = Math.max(...days.map((d) => d.cost), 0.01);

    if (maxCost <= 0) {
      section.style.display = 'none';
      return;
    }
    section.style.display = 'block';
    const chartEl = section.querySelector('.chart-bars') || (() => {
      const d = document.createElement('div');
      d.className = 'chart-bars';
      section.appendChild(d);
      return d;
    })();
    chartEl.innerHTML = days.map((d) => {
      const h = Math.max(2, (d.cost / maxCost) * 100);
      return `<div class="chart-col">
        <div class="chart-bar" style="height:${h}%" title="$${d.cost.toFixed(2)}"></div>
        <span class="chart-label">${d.label.split(' ')[1]}</span>
      </div>`;
    }).join('');
  } catch (e) {
    console.error('usage chart error:', e);
    if (section) section.style.display = 'none';
  }
}

// ── Settings ──────────────────────────────

async function loadSettings() {
  try {
    const s = await window.__TAURI__.core.invoke('get_settings');
    document.getElementById('token-input').value = s.token || '';
    document.getElementById('interval-input').value = s.refresh_minutes || 5;
    document.getElementById('show-text-check').checked = s.show_percent_text !== false;
    document.getElementById('theme-select').value = s.theme || 'system';
    document.getElementById('alert-threshold').value = s.alert_threshold ?? 20;
    document.getElementById('auto-start-check').checked = s.auto_start === true;
  } catch (e) {
    console.error('loadSettings error:', e);
  }
}

document.getElementById('theme-select').addEventListener('change', (e) => {
  const theme = e.target.value;
  document.documentElement.classList.remove('theme-light', 'theme-dark');
  if (theme === 'light') document.documentElement.classList.add('theme-light');
  else if (theme === 'dark') document.documentElement.classList.add('theme-dark');
});

document.getElementById('toggle-token').addEventListener('click', () => {
  const inp = document.getElementById('token-input');
  inp.type = inp.type === 'password' ? 'text' : 'password';
});

document.getElementById('interval-input').addEventListener('blur', (e) => {
  let val = parseInt(e.target.value, 10);
  if (isNaN(val) || val < 1) val = 1;
  if (val > 60) val = 60;
  e.target.value = val;
});

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
    await window.__TAURI__.core.invoke('save_settings', { newSettings: settings });
    await window.__TAURI__.core.invoke('toggle_autostart', { enabled: settings.auto_start }).catch(() => {});
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

// ── Keyboard Shortcuts ────────────────────

document.addEventListener('keydown', async (e) => {
  const mod = e.metaKey || e.ctrlKey;
  if (mod && e.key === 'r') {
    e.preventDefault();
    document.getElementById('last-update').textContent = 'Refreshing…';
    await window.__TAURI__.core.invoke('refresh_now');
  } else if (mod && e.key === 'w') {
    e.preventDefault();
    const { getCurrentWebviewWindow } = window.__TAURI__.webviewWindow;
    getCurrentWebviewWindow().close();
  } else if (mod && e.key === ',') {
    e.preventDefault();
    switchTab('settings');
  }
});

// ── Theme ─────────────────────────────────

async function applyTheme() {
  try {
    const s = await window.__TAURI__.core.invoke('get_settings');
    const theme = s.theme || 'system';
    document.documentElement.classList.remove('theme-light', 'theme-dark');
    if (theme === 'light') {
      document.documentElement.classList.add('theme-light');
    } else if (theme === 'dark') {
      document.documentElement.classList.add('theme-dark');
    }
    // 'system' = no class, relies on prefers-color-scheme
  } catch (e) { /* use system default */ }
}

// ── Init ──────────────────────────────────

window.addEventListener('DOMContentLoaded', async () => {
  await waitForTauri();
  await applyTheme();
  await loadBalance();
  loadModelBreakdown();
  loadUsageChart();
  window.__TAURI__.event.listen('balance-updated', (e) => {
    updateUI(e.payload);
    loadModelBreakdown();
    loadUsageChart();
  });
  window.__TAURI__.event.listen('balance-error', (e) => {
    document.getElementById('last-update').textContent = `Error: ${e.payload}`;
  });
  window.__TAURI__.event.listen('settings-changed', () => {
    applyTheme();
    loadBalance();
    loadModelBreakdown();
    loadUsageChart();
  });
});
