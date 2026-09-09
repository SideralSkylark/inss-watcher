use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{
    app::orchestrator::StatusResponse,
    config::Settings,
    infra::persistence,
};

const WEB_ADDR: &str = "127.0.0.1:8765";

#[derive(Serialize)]
struct ApiResponse<T> {
    ok: bool,
    data: T,
}

#[derive(Deserialize)]
struct ControlRequest {
    action: String,
}

#[derive(Deserialize)]
struct SettingsUpdate {
    match_notifications: Option<bool>,
    failure_notifications: Option<bool>,
    output_path: Option<String>,
    watch_dirs: Option<String>,
}

#[derive(Serialize)]
struct UiSettings {
    output_path: String,
    watch_dirs: String,
    match_notifications: bool,
    failure_notifications: bool,
}

pub fn start(settings: &Settings) -> anyhow::Result<()> {
    let listener = TcpListener::bind(WEB_ADDR)
        .with_context(|| format!("failed to bind local web UI to {}", WEB_ADDR))?;

    info!(addr = WEB_ADDR, "web interface listening");

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(e) => {
                warn!(error = %e, "web UI connection error");
                continue;
            }
        };

        let settings = settings.clone();
        std::thread::spawn(move || handle_connection(stream, settings));
    }

    Ok(())
}

fn handle_connection(stream: TcpStream, settings: Settings) {
    let mut reader = BufReader::new(stream.try_clone().unwrap_or_else(|_| stream.try_clone().unwrap()));
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }

    let parts = request_line.trim_end().split_whitespace().collect::<Vec<_>>();
    if parts.len() < 3 {
        return;
    }

    let method = parts[0];
    let path_and_query = parts[1];

    let mut headers = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            return;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        headers.push_str(&line);
    }

    let body = if method.eq_ignore_ascii_case("POST") {
        let content_length = headers
            .lines()
            .filter_map(|line| line.split_once(':'))
            .find(|(key, _)| key.eq_ignore_ascii_case("Content-Length"))
            .and_then(|(_, value)| value.trim().parse::<usize>().ok())
            .unwrap_or(0);

        let mut bytes = vec![0u8; content_length];
        if content_length > 0 && reader.read_exact(&mut bytes).is_err() {
            return;
        }
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        String::new()
    };

    let (path, query) = split_path_query(path_and_query);

    let response = match (method, path) {
        ("GET", "/") => render_page(),
        ("GET", "/styles.css") => render_css(),
        ("GET", "/app.js") => render_js(),
        ("GET", "/api/status") => api_status(&settings),
        ("GET", "/api/settings") => api_settings(&settings),
        ("GET", "/api/documents") => api_documents(query),
        ("GET", "/api/recent-matches") => api_recent_matches(),
        ("POST", "/api/control") => api_control(&settings, &body),
        ("POST", "/api/settings") => api_save_settings(&body),
        ("POST", "/api/cleanup") => api_cleanup(&body),
        ("POST", "/api/documents/delete") => api_delete_document(&body),
        _ => response_json(404, Some("not found"), "not found"),
    };

    let mut stream = stream;
    let _ = write_response(&mut stream, &response);
}

fn split_path_query(raw: &str) -> (&str, Option<&str>) {
    let mut parts = raw.splitn(2, '?');
    let path = parts.next().unwrap_or("/");
    let query = parts.next();
    (path, query)
}

fn response_json<T: Serialize>(status: u16, data: Option<T>, message: &str) -> Vec<u8> {
    let payload = match data {
        Some(data) => serde_json::to_string(&ApiResponse { ok: status < 400, data }).unwrap_or_else(|_| "{}".to_string()),
        None => serde_json::to_string(&serde_json::json!({ "ok": status < 400, "message": message })).unwrap_or_else(|_| "{}".to_string()),
    };

    let mut out = Vec::new();
    out.extend_from_slice(format!("HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", status, http_status_text(status), payload.len()).as_bytes());
    out.extend_from_slice(payload.as_bytes());
    out
}

fn write_response(stream: &mut TcpStream, body: &[u8]) -> std::io::Result<()> {
    stream.write_all(body)
}

fn http_status_text(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

fn api_status(settings: &Settings) -> Vec<u8> {
    let socket_path = &settings.daemon.socket_path;
    let status = match query_status(socket_path) {
        Ok(status) => status,
        Err(e) => {
            error!(error = %e, "failed to query daemon status for web UI");
            let payload = serde_json::json!({
                "ok": false,
                "message": format!("status unavailable: {e}"),
                "queue_depth": 0,
                "matched": 0,
                "unmatched": 0
            });
            return response_json(500, Some(payload), "status unavailable");
        }
    };

    let payload = serde_json::json!({
        "state": "running",
        "queue_depth": status.queue_depth,
        "matched": status.matched,
        "unmatched": status.unmatched.len(),
        "items": status.unmatched,
    });
    response_json(200, Some(payload), "ok")
}

fn api_settings(settings: &Settings) -> Vec<u8> {
    let payload = UiSettings {
        output_path: settings.storage.output_path.to_string_lossy().to_string(),
        watch_dirs: settings
            .watcher
            .dirs_to_watch
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(", "),
        match_notifications: settings.notifications.match_notifications,
        failure_notifications: settings.notifications.failure_notifications,
    };
    response_json(200, Some(payload), "ok")
}

fn api_documents(query: Option<&str>) -> Vec<u8> {
    let params = parse_query(query.unwrap_or(""));
    let status = params.get("status").map(String::as_str).filter(|s| !s.is_empty());
    let contributor = params.get("contributor").map(String::as_str).filter(|s| !s.is_empty());
    let period = params.get("period").map(String::as_str).filter(|s| !s.is_empty());

    let documents = match persistence::query_documents(status, contributor, period, 50) {
        Ok(rows) => rows,
        Err(e) => {
            error!(error = %e, "failed to query documents for web UI");
            return response_json::<serde_json::Value>(500, None, "database query failed");
        }
    };

    response_json(200, Some(documents), "ok")
}

fn api_recent_matches() -> Vec<u8> {
    let matches = match persistence::recent_matches(10) {
        Ok(items) => items,
        Err(e) => {
            error!(error = %e, "failed to query recent matches for web UI");
            return response_json::<serde_json::Value>(500, None, "database query failed");
        }
    };

    response_json(200, Some(matches), "ok")
}

fn api_control(settings: &Settings, body: &str) -> Vec<u8> {
    let request: ControlRequest = match serde_json::from_str(body) {
        Ok(req) => req,
        Err(_) => return response_json::<serde_json::Value>(400, None, "invalid request body"),
    };

    let action = request.action.trim();
    let command = match action {
        "rescan" | "pause" | "resume" | "stop" => action,
        _ => return response_json::<serde_json::Value>(400, None, "unsupported action"),
    };

    match send_ipc_command(&settings.daemon.socket_path, command) {
        Ok(output) => response_json(200, Some(serde_json::json!({ "message": output })), "ok"),
        Err(e) => response_json(500, Some(serde_json::json!({ "message": format!("{}", e) })), "command failed"),
    }
}

fn api_save_settings(body: &str) -> Vec<u8> {
    let update: SettingsUpdate = match serde_json::from_str(body) {
        Ok(update) => update,
        Err(_) => return response_json::<serde_json::Value>(400, None, "invalid request body"),
    };

    let mut settings = match Settings::load() {
        Ok(settings) => settings,
        Err(e) => return response_json(500, Some(serde_json::json!({"message": format!("{}", e)})), "settings failed"),
    };

    if let Some(value) = update.match_notifications {
        settings.notifications.match_notifications = value;
    }
    if let Some(value) = update.failure_notifications {
        settings.notifications.failure_notifications = value;
    }
    if let Some(path) = update.output_path.filter(|p| !p.trim().is_empty()) {
        settings.storage.output_path = PathBuf::from(path);
    }
    if let Some(dirs) = update.watch_dirs.filter(|value| !value.trim().is_empty()) {
        settings.watcher.dirs_to_watch = dirs
            .split(',')
            .map(str::trim)
            .filter(|dir| !dir.is_empty())
            .map(PathBuf::from)
            .collect();
    }

    let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("inss-watcher/config.toml");

    match toml::to_string_pretty(&settings) {
        Ok(content) => {
            if let Err(e) = fs::write(&config_path, content) {
                return response_json(500, Some(serde_json::json!({"message": format!("{}", e)})), "write failed");
            }
        }
        Err(e) => return response_json(500, Some(serde_json::json!({"message": format!("{}", e)})), "serialize failed"),
    }

    response_json(200, Some(serde_json::json!({ "saved": true })), "ok")
}

fn api_cleanup(body: &str) -> Vec<u8> {
    let action = match serde_json::from_str::<serde_json::Value>(body) {
        Ok(value) => value
            .get("action")
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .unwrap_or_default(),
        Err(_) => return response_json::<serde_json::Value>(400, None, "invalid request body"),
    };

    let removed = match action.as_str() {
        "prune_missing" => match persistence::cleanup_missing_files() {
            Ok(count) => count,
            Err(e) => {
                error!(error = %e, "failed to prune missing files");
                return response_json(500, Some(serde_json::json!({ "message": format!("{}", e) })), "cleanup failed");
            }
        },
        _ => return response_json::<serde_json::Value>(400, None, "unsupported cleanup action"),
    };

    response_json(200, Some(serde_json::json!({ "removed": removed })), "ok")
}

fn api_delete_document(body: &str) -> Vec<u8> {
    let value: serde_json::Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return response_json::<serde_json::Value>(400, None, "invalid request body"),
    };

    let id = match value.get("id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return response_json::<serde_json::Value>(400, None, "missing id"),
    };

    if let Err(e) = persistence::delete_document(id) {
        error!(error = %e, doc_id = id, "failed to delete document from UI");
        return response_json(500, Some(serde_json::json!({ "message": format!("{}", e) })), "delete failed");
    }

    response_json(200, Some(serde_json::json!({ "deleted": true })), "ok")
}

fn parse_query(raw: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    if raw.is_empty() {
        return out;
    }

    for part in raw.split('&') {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        out.insert(key.to_string(), url_decode(value));
    }
    out
}

fn url_decode(value: &str) -> String {
    value.replace("+", " ")
}

fn query_status(socket_path: &std::path::Path) -> anyhow::Result<StatusResponse> {
    let mut stream = std::os::unix::net::UnixStream::connect(socket_path)?;
    let request = serde_json::to_string(&serde_json::json!({ "command": "status" }))?;
    std::io::Write::write_all(&mut stream, request.as_bytes())?;
    std::io::Write::write_all(&mut stream, b"\n")?;
    std::io::Write::flush(&mut stream)?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let _ = reader.read_line(&mut line)?;
    let response: StatusResponse = serde_json::from_str(&line)?;
    Ok(response)
}

fn send_ipc_command(socket_path: &std::path::Path, command: &str) -> anyhow::Result<String> {
    let mut stream = std::os::unix::net::UnixStream::connect(socket_path).map_err(|e| anyhow::anyhow!("daemon unavailable: {e}"))?;
    let request = serde_json::to_string(&serde_json::json!({ "command": command }))?;
    std::io::Write::write_all(&mut stream, request.as_bytes())?;
    std::io::Write::write_all(&mut stream, b"\n")?;
    std::io::Write::flush(&mut stream)?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let _ = reader.read_line(&mut line)?;
    Ok(line.trim().to_string())
}

fn render_page() -> Vec<u8> {
    let html = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>INSS Watcher</title>
    <link rel="stylesheet" href="/styles.css" />
  </head>
  <body>
    <div class="wrap">
      <header class="topbar">
        <div>
          <h1>INSS Watcher</h1>
        </div>
        <div class="actions">
          <button data-action="rescan">Rescan</button>
          <button data-action="pause">Pause</button>
          <button data-action="resume">Resume</button>
        </div>
      </header>

      <main>
        <section class="card overview">
          <h2>Overview</h2>
          <div id="overview"></div>
        </section>

        <section class="card">
          <h2>Recent matches</h2>
          <table>
            <thead>
              <tr><th>Contributor</th><th>Period</th><th>Reference</th></tr>
            </thead>
            <tbody id="recentMatches"></tbody>
          </table>
        </section>

        <section class="card">
          <h2>Documents</h2>
          <div class="filters">
            <select id="filterStatus">
              <option value="">All statuses</option>
              <option value="matched">Matched</option>
              <option value="stored">Unmatched</option>
              <option value="failed">Failed</option>
            </select>
            <input id="filterContributor" type="text" placeholder="Contributor" />
            <input id="filterPeriod" type="text" placeholder="MM/YYYY" />
            <button id="applyFilters">Apply</button>
          </div>
          <table>
            <thead>
              <tr><th>ID</th><th>Type</th><th>Status</th><th>Contributor</th><th>Period</th><th>Reference</th><th>Path</th><th>Action</th></tr>
            </thead>
            <tbody id="documentsTable"></tbody>
          </table>
        </section>

        <section class="card">
          <h2>Database cleanup</h2>
          <button id="pruneMissing">Prune missing files</button>
        </section>

        <section class="card">
          <h2>Settings</h2>
          <form id="settingsForm">
            <label><input type="checkbox" name="match_notifications" /> Match notifications</label>
            <label><input type="checkbox" name="failure_notifications" /> Failure notifications</label>
            <label>Output directory<input name="output_path" type="text" /></label>
            <label>Watch directories<input name="watch_dirs" type="text" /></label>
            <button type="submit">Save settings</button>
          </form>
        </section>
      </main>
    </div>

    <script src="/app.js"></script>
  </body>
</html>"#;
    let mut out = Vec::new();
    out.extend_from_slice(format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", html.len()).as_bytes());
    out.extend_from_slice(html.as_bytes());
    out
}

fn render_css() -> Vec<u8> {
    let css = r#":root { color-scheme: light dark; font-family: sans-serif; }
body { margin: 0; background: #111827; color: #e5e7eb; }
.wrap { max-width: 1100px; margin: 0 auto; padding: 24px; }
.topbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
.actions { display: flex; gap: 12px; }
button { background: #2563eb; color: white; border: 0; border-radius: 6px; padding: 8px 12px; cursor: pointer; }
button.secondary { background: #374151; }
.card { background: rgba(17,24,39,0.9); border-radius: 12px; padding: 18px; margin-bottom: 18px; }
.overview { min-height: 120px; }
.filters { display: flex; gap: 12px; margin-bottom: 16px; flex-wrap: wrap; }
input, select { padding: 8px; border-radius: 6px; border: 1px solid #475569; background: #0f172a; color: white; }
label { display: block; margin: 10px 0; }
table { width: 100%; border-collapse: collapse; }
th, td { text-align: left; padding: 8px; border-bottom: 1px solid #334155; vertical-align: top; }
.code { font-family: monospace; font-size: 12px; }
small { color: #a5b4fc; }
"#;
    let mut out = Vec::new();
    out.extend_from_slice(format!("HTTP/1.1 200 OK\r\nContent-Type: text/css\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", css.len()).as_bytes());
    out.extend_from_slice(css.as_bytes());
    out
}

fn render_js() -> Vec<u8> {
    let js = r#"const state = { status: null, documents: [], recentMatches: [] };

async function request(path, options = {}) {
  const response = await fetch(path, { headers: { 'Content-Type': 'application/json' }, ...options });
  if (!response.ok) {
    throw new Error(await response.text());
  }
  return response.json();
}

function renderOverview(data) {
  const el = document.getElementById('overview');
  if (!el) return;
  const unmatched = data.unmatched ?? 0;
  const matched = data.matched ?? 0;
  el.innerHTML = `
    <div><strong>State:</strong> ${data.state ?? 'unknown'}</div>
    <div><strong>Queue:</strong> ${data.queue_depth ?? 0}</div>
    <div><strong>Matched:</strong> ${matched}</div>
    <div><strong>Unmatched:</strong> ${unmatched}</div>
  `;
}

async function loadStatus() {
  try {
    const data = await request('/api/status');
    state.status = data.data;
    renderOverview(state.status);
  } catch (err) {
    console.error(err);
  }
}

function renderRecentMatches(items) {
  const tbody = document.getElementById('recentMatches');
  if (!tbody) return;
  tbody.innerHTML = (items || []).map(item => `
    <tr>
      <td>${item.contributor_num || '—'}</td>
      <td>${item.ref_period || '—'}</td>
      <td>${item.reference_num || '—'}</td>
    </tr>
  `).join('') || '<tr><td colspan="3">No matches yet</td></tr>';
}

async function loadRecentMatches() {
  try {
    const data = await request('/api/recent-matches');
    state.recentMatches = data.data || [];
    renderRecentMatches(state.recentMatches);
  } catch (err) {
    console.error(err);
  }
}

function renderDocuments(rows) {
  const tbody = document.getElementById('documentsTable');
  if (!tbody) return;
  tbody.innerHTML = (rows || []).map(row => `
    <tr>
      <td>${row.id}</td>
      <td>${row.doc_type}</td>
      <td>${row.status}</td>
      <td>${row.contributor_num || '—'}</td>
      <td>${row.ref_period || '—'}</td>
      <td>${row.reference_num || '—'}</td>
      <td class="code">${(row.path || '').replace(/\//g, '/')}</td>
      <td><button class="secondary" data-delete-id="${row.id}">Delete</button></td>
    </tr>
  `).join('') || '<tr><td colspan="8">No documents found</td></tr>';

  for (const button of document.querySelectorAll('[data-delete-id]')) {
    button.addEventListener('click', async () => {
      const id = Number(button.getAttribute('data-delete-id'));
      await request('/api/documents/delete', { method: 'POST', body: JSON.stringify({ id }) });
      await loadDocuments();
    });
  }
}

async function loadDocuments() {
  const status = document.getElementById('filterStatus').value;
  const contributor = document.getElementById('filterContributor').value.trim();
  const period = document.getElementById('filterPeriod').value.trim();

  const params = new URLSearchParams();
  if (status) params.set('status', status);
  if (contributor) params.set('contributor', contributor);
  if (period) params.set('period', period);

  try {
    const data = await request(`/api/documents?${params.toString()}`);
    renderDocuments(data.data || []);
  } catch (err) {
    console.error(err);
  }
}

async function loadSettings() {
  const data = await request('/api/settings');
  const settings = data.data || {};
  const form = document.getElementById('settingsForm');
  form.match_notifications.checked = !!settings.match_notifications;
  form.failure_notifications.checked = !!settings.failure_notifications;
  form.output_path.value = settings.output_path || '';
  form.watch_dirs.value = settings.watch_dirs || '';
}

async function saveSettings(event) {
  event.preventDefault();
  const form = event.target;
  const payload = {
    match_notifications: form.match_notifications.checked,
    failure_notifications: form.failure_notifications.checked,
    output_path: form.output_path.value.trim(),
    watch_dirs: form.watch_dirs.value.trim(),
  };
  await request('/api/settings', { method: 'POST', body: JSON.stringify(payload) });
  await loadStatus();
}

async function triggerAction(action) {
  await request('/api/control', {
    method: 'POST',
    body: JSON.stringify({ action })
  });
  await loadStatus();
}

async function pruneMissing() {
  await request('/api/cleanup', {
    method: 'POST',
    body: JSON.stringify({ action: 'prune_missing' })
  });
  await loadDocuments();
}

for (const button of document.querySelectorAll('[data-action]')) {
  button.addEventListener('click', () => triggerAction(button.dataset.action));
}

document.getElementById('applyFilters').addEventListener('click', loadDocuments);
document.getElementById('pruneMissing').addEventListener('click', pruneMissing);
document.getElementById('settingsForm').addEventListener('submit', saveSettings);

loadStatus();
loadRecentMatches();
loadDocuments();
loadSettings();
setInterval(() => {
  loadStatus();
  loadRecentMatches();
  loadDocuments();
}, 15000);
"#;
    let mut out = Vec::new();
    out.extend_from_slice(format!("HTTP/1.1 200 OK\r\nContent-Type: application/javascript\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", js.len()).as_bytes());
    out.extend_from_slice(js.as_bytes());
    out
}
