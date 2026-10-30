#[allow(dead_code)]
#[path = "../src/config.rs"]
mod config;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir=std::env::temp_dir().join(format!("pi-workbench-settings-check-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    std::env::set_var("PI_WORKBENCH_CONFIG_DIR", &dir);
    let catalog=json!([{"id":"test","models":[{"id":"model","reasoning":true}]}]);
    let changes=[json!({"theme":"neon"}),json!({"language":"en"}),json!({"thinking":{"provider":"test","model":"model","level":"high"}})];
    let tasks:Vec<_>=changes.into_iter().map(|change|{let catalog=catalog.clone();std::thread::spawn(move||config::save_settings(change,&catalog).unwrap())}).collect();
    for task in tasks {task.join().unwrap();}
    let result=config::settings();
    assert_eq!(result["theme"],"neon");
    assert_eq!(result["language"],"en");
    assert_eq!(result["thinking"]["test/model"],"high");
    std::fs::remove_file(dir.join("settings.json"))?;
    std::fs::remove_file(dir.join("settings.lock"))?;
    std::fs::remove_dir(dir)?;
    println!("concurrent_settings_updates_preserved=true");
    Ok(())
}
