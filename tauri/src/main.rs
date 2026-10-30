#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod config;
mod external_link;
#[cfg(windows)]
mod session_job;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use rand::{distributions::Alphanumeric, Rng};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Clone)]
struct AppState {
    token: String,
    origin: String,
    catalog: Arc<Mutex<Value>>,
    app: tauri::AppHandle,
}
fn authorized(h: &HeaderMap, s: &AppState) -> bool {
    h.get("x-workbench-token").and_then(|v| v.to_str().ok()) == Some(s.token.as_str())
}
fn error(code: StatusCode, text: &str) -> Response {
    (code, Json(json!({"error":text}))).into_response()
}
async fn assets(uri: axum::http::Uri) -> Response {
    let (mime, body) = match uri.path() {
        "/" => (
            "text/html; charset=utf-8",
            include_str!("../assets/index.html"),
        ),
        "/client.js" => (
            "text/javascript; charset=utf-8",
            include_str!("../assets/client.js"),
        ),
        "/bridge.js" => (
            "text/javascript; charset=utf-8",
            include_str!("../assets/bridge.js"),
        ),
        "/ui-controls.js" => (
            "text/javascript; charset=utf-8",
            include_str!("../assets/ui-controls.js"),
        ),
        "/style.css" => (
            "text/css; charset=utf-8",
            include_str!("../assets/style.css"),
        ),
        "/xterm.js" => (
            "text/javascript; charset=utf-8",
            include_str!("../assets/xterm.js"),
        ),
        "/xterm.css" => (
            "text/css; charset=utf-8",
            include_str!("../assets/xterm.css"),
        ),
        "/fit.js" => (
            "text/javascript; charset=utf-8",
            include_str!("../assets/fit.js"),
        ),
        "/web-links.js" => (
            "text/javascript; charset=utf-8",
            include_str!("../assets/web-links.js"),
        ),
        "/locales/en.json" => (
            "application/json",
            include_str!("../assets/locales/en.json"),
        ),
        "/locales/zh-CN.json" => (
            "application/json",
            include_str!("../assets/locales/zh-CN.json"),
        ),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    ([(header::CONTENT_TYPE,mime),(header::CACHE_CONTROL,"no-store"),(header::X_CONTENT_TYPE_OPTIONS,"nosniff"),(header::CONTENT_SECURITY_POLICY,"default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self'")],body).into_response()
}
async fn bootstrap(State(s): State<AppState>, h: HeaderMap) -> Response {
    if !authorized(&h, &s) {
        return error(StatusCode::FORBIDDEN, "无权访问");
    }
    Json(json!({"settings":config::settings(),"shortcuts":config::shortcuts(),"version":env!("CARGO_PKG_VERSION")})).into_response()
}
async fn save_shortcuts(State(s): State<AppState>, h: HeaderMap, Json(v): Json<Value>) -> Response {
    if !authorized(&h, &s) || h.get(header::ORIGIN).and_then(|v| v.to_str().ok()) != Some(s.origin.as_str()) {
        return error(StatusCode::FORBIDDEN, "无权访问");
    }
    let catalog = s.catalog.lock().unwrap().clone();
    match tokio::task::spawn_blocking(move || config::update_shortcuts(v, &catalog)).await {
        Ok(Ok(data)) => Json(data).into_response(),
        Ok(Err(message)) => error(StatusCode::BAD_REQUEST, &message),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "无法保存设置"),
    }
}
async fn catalog(State(s): State<AppState>, h: HeaderMap) -> Response {
    if !authorized(&h, &s) {
        return error(StatusCode::FORBIDDEN, "无权访问");
    }
    let (models, msg) = match tokio::task::spawn_blocking(config::catalog).await {
        Ok(Ok(v)) => (v, ""),
        _ => (
            json!([]),
            "无法读取 Pi 模型，请确认已安装 Pi 并配置供应商。",
        ),
    };
    *s.catalog.lock().unwrap() = models.clone();
    Json(json!({"providers":models,"error":msg,"settings":config::settings(),"version":env!("CARGO_PKG_VERSION")})).into_response()
}
async fn save(State(s): State<AppState>, h: HeaderMap, Json(v): Json<Value>) -> Response {
    if !authorized(&h, &s) {
        return error(StatusCode::FORBIDDEN, "无权访问");
    }
    let catalog = s.catalog.lock().unwrap().clone();
    match tokio::task::spawn_blocking(move || config::save_settings(v, &catalog)).await {
        Ok(Ok(v)) => Json(v).into_response(),
        Ok(Err(e)) => error(StatusCode::BAD_REQUEST, &e),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "无法保存设置"),
    }
}
async fn thinking(
    State(s): State<AppState>,
    h: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    if !authorized(&h, &s) {
        return error(StatusCode::FORBIDDEN, "无权访问");
    }
    let cat = s.catalog.lock().unwrap().clone();
    let selected = match (q.get("provider"), q.get("model")) {
        (None, None) => None,
        (Some(p), Some(m)) if config::model_exists(&cat, p, m, false) => {
            Some((p.clone(), m.clone()))
        }
        _ => return error(StatusCode::BAD_REQUEST, "无效的模型"),
    };
    Json(
        tokio::task::spawn_blocking(move || config::thinking(cat, selected))
            .await
            .unwrap_or(json!({})),
    )
    .into_response()
}
async fn quota(State(s): State<AppState>, h: HeaderMap) -> Response {
    if !authorized(&h, &s) {
        return error(StatusCode::FORBIDDEN, "无权访问");
    }
    let cat = s.catalog.lock().unwrap().clone();
    Json(config::quotas(cat).await).into_response()
}
async fn diagnostics(State(s): State<AppState>, h: HeaderMap) -> Response {
    if !authorized(&h, &s) { return error(StatusCode::FORBIDDEN, "无权访问"); }
    let cat = s.catalog.lock().unwrap().clone();
    let result = tokio::task::spawn_blocking(move || {
        fn available(name: &str) -> bool {
            #[cfg(windows)]
            let mut command = { let mut c = std::process::Command::new("where.exe"); use std::os::windows::process::CommandExt; c.creation_flags(0x08000000); c };
            #[cfg(not(windows))]
            let mut command = std::process::Command::new("which");
            command.arg(name).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().map(|s| s.success()).unwrap_or(false)
        }
        let settings = config::settings();
        let workdir = settings["workdir"].as_str().unwrap_or("");
        json!({"pi":available("pi"),"node":available("node"),"workdir":std::path::Path::new(workdir).is_dir(),"models":cat.as_array().map(|v| v.iter().map(|p| p["models"].as_array().map_or(0, |m| m.len())).sum::<usize>()).unwrap_or(0),"platform":std::env::consts::OS})
    }).await;
    match result { Ok(data) => Json(data).into_response(), Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "诊断失败") }
}
async fn clipboard(State(s): State<AppState>, h: HeaderMap) -> Response {
    if !authorized(&h, &s) || h.get(header::ORIGIN).and_then(|v| v.to_str().ok()) != Some(s.origin.as_str()) {
        return error(StatusCode::FORBIDDEN, "无权访问");
    }
    match tokio::task::spawn_blocking(|| arboard::Clipboard::new().and_then(|mut c| c.get_text())).await {
        Ok(Ok(text)) => Json(json!({"text": text})).into_response(),
        _ => error(StatusCode::BAD_REQUEST, "无法读取剪贴板"),
    }
}
async fn open_link(State(s): State<AppState>, h: HeaderMap, Json(v): Json<Value>) -> Response {
    if !authorized(&h, &s) || h.get(header::ORIGIN).and_then(|v| v.to_str().ok()) != Some(s.origin.as_str()) {
        return error(StatusCode::FORBIDDEN, "无权访问");
    }
    let url = match v["url"].as_str().and_then(|value| external_link::validate(value).ok()) {
        Some(url) => url,
        None => return error(StatusCode::BAD_REQUEST, "不支持的链接"),
    };
    match tokio::task::spawn_blocking(move || external_link::open(&url)).await {
        Ok(Ok(())) => Json(json!({"ok": true})).into_response(),
        _ => error(StatusCode::BAD_REQUEST, "无法打开链接"),
    }
}
async fn window_action(State(s): State<AppState>, h: HeaderMap, Json(v): Json<Value>) -> Response {
    if !authorized(&h, &s)
        || h.get(header::ORIGIN).and_then(|v| v.to_str().ok()) != Some(s.origin.as_str())
    {
        return error(StatusCode::FORBIDDEN, "无权访问");
    }
    if let Some(w) = s.app.get_webview_window("main") {
        match v["action"].as_str().unwrap_or("") {
            "drag" => {
                let _ = w.start_dragging();
            }
            "minimize" => {
                let _ = w.minimize();
            }
            "maximize" => {
                if w.is_maximized().unwrap_or(false) {
                    let _ = w.unmaximize();
                } else {
                    let _ = w.maximize();
                }
            }
            "close" => {
                let _ = w.close();
            }
            _ => return error(StatusCode::BAD_REQUEST, "无效操作"),
        }
    }
    Json(json!({"ok":true})).into_response()
}
async fn terminal(
    State(s): State<AppState>,
    h: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    ws: WebSocketUpgrade,
) -> Response {
    let p = q.get("provider").cloned().unwrap_or_default();
    let m = q.get("model").cloned().unwrap_or_default();
    if q.get("token") != Some(&s.token)
        || h.get(header::ORIGIN).and_then(|v| v.to_str().ok()) != Some(s.origin.as_str())
        || !config::model_exists(&s.catalog.lock().unwrap(), &p, &m, false)
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    ws.on_upgrade(move |socket| terminal_session(socket, p, m))
}
async fn terminal_session(mut socket: WebSocket, provider: String, model: String) {
    let settings = config::settings();
    let cwd = settings["workdir"].as_str().unwrap_or("");
    if !std::path::Path::new(cwd).is_absolute() || !std::path::Path::new(cwd).is_dir() {
        return;
    }
    let key = format!("{provider}/{model}");
    let level = settings["thinking"][&key].as_str().unwrap_or("default");
    let pty = match native_pty_system().openpty(PtySize {
        rows: 30,
        cols: 100,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(v) => v,
        Err(_) => return,
    };
    #[cfg(windows)]
    let mut cmd = {
        let mut c = CommandBuilder::new("powershell.exe");
        let quote = |s: &str| format!("'{}'", s.replace('\'', "''"));
        let mut args = vec![
            "pi".to_string(),
            "--provider".into(),
            provider.clone(),
            "--model".into(),
            model.clone(),
        ];
        if config::LEVELS.contains(&level) && level != "default" {
            args.extend(["--thinking".into(), level.into()]);
        }
        c.args([
            "-NoLogo",
            "-NoExit",
            "-Command",
            &format!(
                "& {}",
                args.iter().map(|x| quote(x)).collect::<Vec<_>>().join(" ")
            ),
        ]);
        c
    };
    #[cfg(not(windows))]
    let mut cmd = {
        let mut c = CommandBuilder::new("pi");
        c.args(["--provider", &provider, "--model", &model]);
        if config::LEVELS.contains(&level) && level != "default" {
            c.args(["--thinking", level]);
        }
        c
    };
    cmd.cwd(cwd);
    cmd.env("TERM", "xterm-256color");
    cmd.env("PI_HARDWARE_CURSOR", "1");
    #[cfg(windows)]
    let job = match session_job::SessionJob::new() {
        Ok(job) => job,
        Err(_) => {
            let _ = socket.send(Message::Text("\r\nSession startup failed: unable to create process group. / 无法创建会话进程组。\r\n".into())).await;
            return;
        }
    };
    let mut child = match pty.slave.spawn_command(cmd) {
        Ok(v) => v,
        Err(_) => return,
    };
    #[cfg(windows)]
    if job.attach(child.as_ref()).is_err() {
        let _ = child.kill();
        let _ = child.wait();
        let _ = socket.send(Message::Text("\r\nSession startup failed: unable to manage session processes. / 无法管理会话进程。\r\n".into())).await;
        return;
    }
    drop(pty.slave);
    let mut reader = match pty.master.try_clone_reader() {
        Ok(v) => v,
        Err(_) => return,
    };
    let mut writer = match pty.master.take_writer() {
        Ok(v) => v,
        Err(_) => return,
    };
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(64);
    std::thread::spawn(move || {
        let mut buf = [0_u8; 8192];
        let mut pending = Vec::new();
        loop {
            let n = match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };
            pending.extend_from_slice(&buf[..n]);
            let consumed = match std::str::from_utf8(&pending) {
                Ok(v) => {
                    let _ = tx.blocking_send(v.to_owned());
                    pending.len()
                }
                Err(e) => {
                    let valid = e.valid_up_to();
                    if valid > 0 {
                        let _ = tx
                            .blocking_send(String::from_utf8_lossy(&pending[..valid]).into_owned());
                    }
                    if let Some(n) = e.error_len() {
                        let _ = tx.blocking_send("�".into());
                        valid + n
                    } else {
                        valid
                    }
                }
            };
            pending.drain(..consumed);
        }
    });
    let (mut out, mut input) = socket.split();
    loop {
        tokio::select! {
            data=rx.recv()=>match data{Some(v)=>if out.send(Message::Text(v.into())).await.is_err(){break},None=>break},
            event=input.next()=>match event{Some(Ok(Message::Text(v)))=>if let Ok(v)=serde_json::from_str::<Value>(&v){match v["type"].as_str(){Some("input")=>if let Some(data)=v["data"].as_str(){let _=writer.write_all(data.as_bytes());},Some("resize")=>{let cols=v["cols"].as_u64().unwrap_or(0);let rows=v["rows"].as_u64().unwrap_or(0);if cols>0&&cols<500&&rows>0&&rows<500{let _=pty.master.resize(PtySize{rows:rows as u16,cols:cols as u16,pixel_width:0,pixel_height:0});}},_=>{}}},Some(Ok(Message::Close(_)))|Some(Err(_))|None=>break,_=>{}}
        }
    }
    #[cfg(windows)]
    drop(job);
    let _ = child.kill();
    let _ = child.wait();
}
fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("local listener");
    listener.set_nonblocking(true).unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect();
    tauri::Builder::default()
        .setup(move |app| {
            let s = AppState {
                token: token.clone(),
                origin: origin.clone(),
                catalog: Arc::new(Mutex::new(json!([]))),
                app: app.handle().clone(),
            };
            let router = Router::new()
                .route("/api/bootstrap", get(bootstrap))
                .route("/api/catalog", get(catalog))
                .route("/api/settings", post(save))
                .route("/api/model-shortcuts", post(save_shortcuts))
                .route("/api/thinking-levels", get(thinking))
                .route("/api/quota", get(quota))
                .route("/api/diagnostics", get(diagnostics))
                .route("/api/clipboard", post(clipboard))
                .route("/api/open-link", post(open_link))
                .route("/api/window", post(window_action))
                .route("/terminal", get(terminal))
                .fallback(assets)
                .with_state(s);
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let l = tokio::net::TcpListener::from_std(listener).unwrap();
                    axum::serve(l, router).await.unwrap();
                });
            });
            let url: tauri::Url = format!("{origin}/#{token}").parse()?;
            let allowed = origin.clone();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("Pi Workbench")
                .inner_size(1400.0, 900.0)
                .min_inner_size(620.0, 420.0)
                .decorations(false)
                .disable_drag_drop_handler()
                .on_navigation(move |url| url.as_str().starts_with(&(allowed.clone() + "/")))
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Pi Workbench");
}
