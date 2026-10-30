use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

pub const LEVELS: &[&str] = &[
    "default", "off", "minimal", "low", "medium", "high", "xhigh", "max",
];
const THEMES: &[&str] = &["mint", "jilan", "dongqing", "mist", "neon", "aurora"];
const LANGUAGES: &[&str] = &["zh-CN", "en"];

fn read_json(p: &Path) -> Value {
    std::fs::read(p)
        .ok()
        .and_then(|v| serde_json::from_slice(&v).ok())
        .unwrap_or(json!({}))
}
fn config_dir() -> PathBuf {
    std::env::var_os("PI_WORKBENCH_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".pi-workbench-v2"))
}
fn settings_path() -> PathBuf {
    config_dir().join("settings.json")
}
fn initial_source(path: &Path, name: &str) -> PathBuf {
    initial_source_in(path, name, &dirs::home_dir().unwrap_or_default(), std::env::var_os("PI_WORKBENCH_CONFIG_DIR").is_some())
}
fn initial_source_in(path: &Path, name: &str, home: &Path, custom: bool) -> PathBuf {
    if path.exists() || custom {
        return path.to_path_buf();
    }
    let prototype = home.join(".pi-workbench-tauri-prototype").join(name);
    if prototype.exists() { return prototype; }
    if name == "settings.json" {
        let legacy = home.join(".pi-workbench").join(name);
        if legacy.exists() { return legacy; }
    }
    path.to_path_buf()
}

#[cfg(test)]
mod migration_tests {
    use super::initial_source_in;
    #[test]
    fn fallback_respects_custom_path_and_existing_v2_files() {
        let home = std::env::temp_dir().join(format!("pi-workbench-migration-{}", std::process::id()));
        let prototype = home.join(".pi-workbench-tauri-prototype");
        let legacy = home.join(".pi-workbench");
        let v2 = home.join(".pi-workbench-v2");
        for dir in [&prototype, &legacy, &v2] { std::fs::create_dir_all(dir).unwrap(); }
        let name = "settings.json";
        let target = v2.join(name);
        std::fs::write(legacy.join(name), b"legacy").unwrap();
        assert_eq!(initial_source_in(&target, name, &home, false), legacy.join(name));
        std::fs::write(prototype.join(name), b"prototype").unwrap();
        assert_eq!(initial_source_in(&target, name, &home, false), prototype.join(name));
        assert_eq!(initial_source_in(&target, name, &home, true), target);
        std::fs::write(&target, b"v2").unwrap();
        assert_eq!(initial_source_in(&target, name, &home, false), target);
        for file in [legacy.join(name), prototype.join(name), target] { std::fs::remove_file(file).unwrap(); }
        for dir in [prototype, legacy, v2, home] { std::fs::remove_dir(dir).unwrap(); }
    }
}
fn shortcuts_path() -> PathBuf {
    if let Some(path) = std::env::var_os("PI_WORKBENCH_SHORTCUTS_FILE") { return PathBuf::from(path); }
    settings_path().with_file_name("model-shortcuts.json")
}
pub fn shortcuts() -> Value {
    let path = shortcuts_path();
    let source = if std::env::var_os("PI_WORKBENCH_SHORTCUTS_FILE").is_some() { path.clone() } else { initial_source(&path, "model-shortcuts.json") };
    let mut data = read_json(&source);
    if !data.is_object() { data = json!({}); }
    for name in ["favorites", "recentModels"] {
        if !data[name].is_array() { data[name] = json!([]); }
    }
    data
}

pub fn update_shortcuts(change: Value, catalog: &Value) -> Result<Value, String> {
    let key = change["key"].as_str().ok_or("无效的模型")?;
    let pair: Vec<String> = serde_json::from_str(key).map_err(|_| "无效的模型")?;
    if pair.len() != 2 || !model_exists(catalog, &pair[0], &pair[1], false) { return Err("无效的模型".into()); }
    let action = change["action"].as_str().unwrap_or("");
    if action != "favorite" && action != "recent" { return Err("无效设置".into()); }
    let enabled = if action == "favorite" { Some(change["enabled"].as_bool().ok_or("无效设置")?) } else { None };
    let path = shortcuts_path();
    let parent = path.parent().ok_or("无法保存设置")?;
    std::fs::create_dir_all(parent).map_err(|_| "无法保存设置")?;
    let lock = std::fs::OpenOptions::new().read(true).write(true).create(true).open(path.with_extension("lock")).map_err(|_| "无法保存设置")?;
    lock.lock().map_err(|_| "无法保存设置")?;
    let mut data = shortcuts();
    let name = if action == "favorite" { "favorites" } else { "recentModels" };
    let list = data[name].as_array_mut().unwrap();
    list.retain(|x| x.as_str() != Some(key));
    if action == "favorite" && enabled == Some(true) && list.len() >= 50 { return Err("收藏数量已达上限".into()); }
    if enabled != Some(false) {
        if action == "favorite" { list.push(json!(key)); } else { list.insert(0, json!(key)); }
    }
    let max = if action == "favorite" { 50 } else { 5 };
    if list.len() > max { list.truncate(max); }
    let temp = path.with_extension(format!("{}.tmp", std::process::id()));
    std::fs::write(&temp, serde_json::to_vec_pretty(&data).map_err(|_| "无法保存设置")?).map_err(|_| "无法保存设置")?;
    replace_file(&temp, &path).map_err(|_| "无法保存设置")?;
    Ok(data)
}

#[cfg(windows)]
fn replace_file(from: &Path, to: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH};
    let source: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let target: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    if unsafe { MoveFileExW(source.as_ptr(), target.as_ptr(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH) } == 0 { Err(std::io::Error::last_os_error()) } else { Ok(()) }
}
#[cfg(not(windows))]
fn replace_file(from: &Path, to: &Path) -> std::io::Result<()> { std::fs::rename(from, to) }
fn pi_dir() -> PathBuf {
    std::env::var_os("PI_CODING_AGENT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_default()
                .join(".pi")
                .join("agent")
        })
}

pub fn settings() -> Value {
    let path = settings_path();
    let mut data = read_json(&initial_source(&path, "settings.json"));
    if !data.is_object() {
        data = json!({});
    }
    if !THEMES.contains(&data["theme"].as_str().unwrap_or("")) {
        data["theme"] = json!("mint");
    }
    if !LANGUAGES.contains(&data["language"].as_str().unwrap_or("")) {
        data["language"] = Value::Null;
    }
    if !data["workdir"]
        .as_str()
        .map(|s| Path::new(s).is_absolute())
        .unwrap_or(false)
    {
        data["workdir"] = json!(dirs::home_dir().unwrap_or_default());
    }
    if !data["thinking"].is_object() {
        data["thinking"] = json!({});
    }
    data
}

pub fn save_settings(change: Value, catalog: &Value) -> Result<Value, String> {
    let path = settings_path();
    std::fs::create_dir_all(path.parent().ok_or("无法保存设置")?).map_err(|_| "无法保存设置")?;
    let lock = std::fs::OpenOptions::new().read(true).write(true).create(true).open(path.with_extension("lock")).map_err(|_| "无法保存设置")?;
    lock.lock().map_err(|_| "无法保存设置")?;
    let mut data = settings();
    if let Some(v) = change.get("workdir") {
        let s = v.as_str().ok_or("请输入绝对路径")?;
        if !Path::new(s).is_absolute() || !Path::new(s).is_dir() {
            return Err("目录不存在或无法访问".into());
        }
        data["workdir"] = json!(s);
    }
    for (name, allowed) in [("theme", THEMES), ("language", LANGUAGES)] {
        if let Some(v) = change.get(name) {
            if !allowed.contains(&v.as_str().unwrap_or("")) {
                return Err("无效设置".into());
            }
            data[name] = v.clone();
        }
    }
    if let Some(v) = change.get("thinking") {
        let p = v["provider"].as_str().unwrap_or("");
        let m = v["model"].as_str().unwrap_or("");
        let level = v["level"].as_str().unwrap_or("");
        if !LEVELS.contains(&level) || !model_exists(catalog, p, m, true) {
            return Err("无效的模型或思考程度".into());
        }
        let key = format!("{p}/{m}");
        let map = data["thinking"].as_object_mut().unwrap();
        if level == "default" {
            map.remove(&key);
        } else {
            map.insert(key, json!(level));
        }
    }
    let temp = path.with_extension(format!("{}.tmp", std::process::id()));
    std::fs::write(&temp, serde_json::to_vec_pretty(&data).unwrap()).map_err(|_| "无法保存设置")?;
    replace_file(&temp, &path).map_err(|_| "无法保存设置")?;
    Ok(data)
}

fn safe_arg(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "_.-/:@+".contains(c))
}
fn pi_command(args: &[&str]) -> Result<Command, String> {
    if args.iter().any(|x| !safe_arg(x)) {
        return Err("不支持的 Pi 参数".into());
    }
    #[cfg(windows)]
    let mut command = {
        let mut c = Command::new("cmd.exe");
        c.args(["/d", "/s", "/c", &format!("pi.cmd {}", args.join(" "))]);
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x08000000);
        c
    };
    #[cfg(not(windows))]
    let mut command = {
        let mut c = Command::new("pi");
        c.args(args);
        c
    };
    command.stderr(Stdio::null());
    Ok(command)
}
fn pi_output(args: &[&str]) -> Option<String> {
    let out = pi_command(args).ok()?.output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn catalog() -> Result<Value, String> {
    let output =
        pi_output(&["--list-models"]).ok_or("无法读取 Pi 模型，请确认已安装 Pi 并配置供应商。")?;
    let mut groups = BTreeMap::<String, Vec<Value>>::new();
    for line in output.lines() {
        let c: Vec<_> = line.split_whitespace().collect();
        if c.len() == 6
            && ["yes", "no"].contains(&c[4].to_lowercase().as_str())
            && ["yes", "no"].contains(&c[5].to_lowercase().as_str())
        {
            groups
                .entry(c[0].into())
                .or_default()
                .push(json!({"id": c[1], "reasoning": c[4].eq_ignore_ascii_case("yes")}));
        }
    }
    Ok(Value::Array(
        groups
            .into_iter()
            .map(|(id, models)| json!({"label":id,"id":id,"models":models}))
            .collect(),
    ))
}
pub fn model_exists(c: &Value, p: &str, m: &str, reasoning: bool) -> bool {
    c.as_array().into_iter().flatten().any(|x| {
        x["id"] == p
            && x["models"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|v| v["id"] == m && (!reasoning || v["reasoning"] == true))
    })
}

pub fn thinking(catalog: Value, selected: Option<(String, String)>) -> Value {
    let settings = settings();
    let mut map = serde_json::Map::new();
    let mut tasks = Vec::new();
    for p in catalog.as_array().into_iter().flatten() {
        for m in p["models"].as_array().into_iter().flatten() {
            let (pid, mid) = (
                p["id"].as_str().unwrap_or(""),
                m["id"].as_str().unwrap_or(""),
            );
            if m["reasoning"] != true
                || selected
                    .as_ref()
                    .map(|(p, m)| p != pid || m != mid)
                    .unwrap_or(false)
            {
                continue;
            }
            tasks.push((pid.to_string(), mid.to_string()));
        }
    }
    for batch in tasks.chunks(4) {
        std::thread::scope(|scope| {
            let handles: Vec<_> = batch
                .iter()
                .map(|(pid, mid)| {
                    let settings = &settings;
                    scope.spawn(move || {
                        let key = format!("{pid}/{mid}");
                        let level = settings["thinking"][&key].as_str().unwrap_or("default");
                        let mut args = vec![
                            "--mode",
                            "rpc",
                            "--provider",
                            pid.as_str(),
                            "--model",
                            mid.as_str(),
                            "--no-session",
                            "--no-extensions",
                            "--no-skills",
                            "--no-mcp",
                            "--offline",
                        ];
                        if level != "default" {
                            args.extend(["--thinking", level]);
                        }
                        let result = (|| -> Option<Value> {
                            let mut cmd = pi_command(&args).ok()?;
                            cmd.current_dir(settings["workdir"].as_str()?)
                                .stdin(Stdio::piped())
                                .stdout(Stdio::piped());
                            let mut child = cmd.spawn().ok()?;
                            child
                                .stdin
                                .take()?
                                .write_all(
                                    b"{\"id\":\"workbench-thinking\",\"type\":\"get_state\"}\n",
                                )
                                .ok()?;
                            let reader = BufReader::new(child.stdout.take()?);
                            let (tx, rx) = std::sync::mpsc::channel();
                            std::thread::spawn(move || {
                                for line in reader.lines().map_while(Result::ok) {
                                    if let Ok(v) = serde_json::from_str::<Value>(&line) {
                                        if v["id"] == "workbench-thinking" && v["success"] == true {
                                            let _ = tx.send(v["data"]["thinkingLevel"].clone());
                                            break;
                                        }
                                    }
                                }
                            });
                            let result = rx.recv_timeout(Duration::from_secs(12)).ok();
                            let _ = child.kill();
                            let _ = child.wait();
                            result
                        })();
                        (key, result.unwrap_or(Value::Null))
                    })
                })
                .collect();
            for handle in handles {
                if let Ok((key, value)) = handle.join() {
                    map.insert(key, value);
                }
            }
        });
    }
    Value::Object(map)
}

pub async fn quotas(catalog: Value) -> Value {
    let mut result = serde_json::Map::new();
    let ids: Vec<String> = catalog
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| p["id"].as_str().map(str::to_owned))
        .collect();
    let values = futures_util::future::join_all(ids.iter().map(|id| quota(id))).await;
    for (id, value) in ids.into_iter().zip(values) {
        result.insert(id, value);
    }
    Value::Object(result)
}
async fn quota(id: &str) -> Value {
    let custom = read_json(&pi_dir().join("models.json"));
    let host = custom["providers"][id]["baseUrl"]
        .as_str()
        .and_then(|s| url::Url::parse(s).ok())
        .and_then(|u| u.host_str().map(str::to_lowercase));
    let kind = match host.as_deref() {
        Some("open.bigmodel.cn") => "zhipu-cn",
        Some("api.z.ai") => "zhipu-global",
        Some("api.minimaxi.com" | "www.minimaxi.com") => "minimax-cn",
        Some("api.minimax.io") => "minimax-global",
        Some("api.deepseek.com") => "deepseek",
        Some(_) => "",
        None => match id {
            "minimax-cn" => "minimax-cn",
            "minimax" => "minimax-global",
            "zai-coding-cn" => "zhipu-cn",
            "zai" | "zai-coding" => "zhipu-global",
            "deepseek" => "deepseek",
            _ => "",
        },
    };
    if kind.is_empty() {
        return json!({"error":"该供应商暂不支持额度查询"});
    }
    let provider = id.to_string();
    let key = tokio::task::spawn_blocking(move || {
        pi_output(&["auth", "print-api-key", "--provider", &provider])
    })
    .await
    .ok()
    .flatten()
    .unwrap_or_default();
    let key = key.trim();
    if key.is_empty() {
        return json!({"error":"Pi 未提供可用于额度查询的 API Key"});
    }
    let (endpoint, auth) = match kind {
        "minimax-cn" => (
            "https://api.minimaxi.com/v1/api/openplatform/coding_plan/remains",
            format!("Bearer {key}"),
        ),
        "minimax-global" => (
            "https://api.minimax.io/v1/api/openplatform/coding_plan/remains",
            format!("Bearer {key}"),
        ),
        "zhipu-cn" => (
            "https://open.bigmodel.cn/api/monitor/usage/quota/limit",
            key.to_string(),
        ),
        "zhipu-global" => (
            "https://api.z.ai/api/monitor/usage/quota/limit",
            key.to_string(),
        ),
        _ => (
            "https://api.deepseek.com/user/balance",
            format!("Bearer {key}"),
        ),
    };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .unwrap();
    let response = match client
        .get(endpoint)
        .header("Authorization", auth)
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return json!({"error":"网络或配置异常"}),
    };
    if !response.status().is_success() {
        return json!({"error":format!("接口返回 HTTP {}", response.status().as_u16())});
    }
    let body: Value = response.json().await.unwrap_or(Value::Null);
    if kind == "deepseek" {
        let balance: Vec<Value> = body["balance_infos"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|x| json!({"currency":x["currency"],"total":x["total_balance"]}))
            .collect();
        return if balance.is_empty() {
            json!({"error":"未返回余额"})
        } else {
            json!({"balance":balance})
        };
    }
    let mut tiers = Vec::new();
    if kind.starts_with("minimax") {
        if let Some(x) = body["model_remains"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|x| x["model_name"] == "general")
        {
            if let Some(v) = x["current_interval_remaining_percent"].as_f64() {
                tiers.push(json!({"name":"5 小时","used":100.0-v,"reset":x["end_time"]}));
            }
            if x["current_weekly_status"] == 1 {
                if let Some(v) = x["current_weekly_remaining_percent"].as_f64() {
                    tiers.push(json!({"name":"本周","used":100.0-v,"reset":x["weekly_end_time"]}));
                }
            }
        }
    } else {
        for x in body["data"]["limits"].as_array().into_iter().flatten() {
            if ["TOKENS_LIMIT", "CREDIT_LIMIT"].contains(&x["type"].as_str().unwrap_or(""))
                && x["percentage"].is_number()
            {
                let name = match x["unit"].as_i64().unwrap_or(0) {
                    3 => "5 小时".to_string(),
                    6 => "本周".to_string(),
                    7 => "本月".to_string(),
                    n => format!("周期 {n}"),
                };
                tiers.push(json!({"name":name,"used":x["percentage"],"reset":x["nextResetTime"]}));
            }
        }
    }
    if tiers.is_empty() {
        json!({"error":"接口未返回套餐额度"})
    } else {
        json!({"tiers":tiers})
    }
}
