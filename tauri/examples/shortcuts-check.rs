#[allow(dead_code)]
#[path = "../src/config.rs"]
mod config;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join(format!("pi-workbench-shortcuts-test-{}.json", std::process::id()));
    std::env::set_var("PI_WORKBENCH_SHORTCUTS_FILE", &path);
    let models: Vec<_> = (0..20).map(|i| json!({"id":format!("model-{i}"),"reasoning":false})).collect();
    let catalog = json!([{"id":"test", "models":models}]);
    let threads: Vec<_> = (0..20).map(|i| { let catalog = catalog.clone(); std::thread::spawn(move || {
        let key = serde_json::to_string(&["test", &format!("model-{i}")]).unwrap();
        config::update_shortcuts(json!({"action":"favorite","key":key,"enabled":true}), &catalog).unwrap();
    }) }).collect();
    for thread in threads { thread.join().unwrap(); }
    assert_eq!(config::shortcuts()["favorites"].as_array().unwrap().len(), 20);
    let key = serde_json::to_string(&["test", "model-0"])?;
    config::update_shortcuts(json!({"action":"recent","key":key}), &catalog)?;
    config::update_shortcuts(json!({"action":"favorite","key":key,"enabled":false}), &catalog)?;
    let read_back = config::shortcuts();
    assert_eq!(read_back["favorites"].as_array().unwrap().len(), 19);
    assert_eq!(read_back["recentModels"].as_array().unwrap().len(), 1);
    assert!(config::update_shortcuts(json!({"action":"favorite","key":"[\"test\",\"invalid\"]","enabled":true}), &catalog).is_err());
    std::fs::remove_file(&path)?;
    std::fs::remove_file(path.with_extension("lock"))?;
    println!("persist_reload=true concurrent_additions=20 invalid_model_rejected=true");
    Ok(())
}
