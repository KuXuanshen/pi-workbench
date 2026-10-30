#[allow(dead_code)]
#[path = "../src/config.rs"]
mod config;
use serde_json::json;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os().nth(2).ok_or("missing test path")?;
    std::env::set_var("PI_WORKBENCH_SHORTCUTS_FILE", path);
    let key = serde_json::to_string(&["test", "model"])?;
    match std::env::args().nth(1).as_deref() {
        Some("write") => {
            let catalog=json!([{"id":"test","models":[{"id":"model","reasoning":false}]}]);
            config::update_shortcuts(json!({"action":"favorite","key":key,"enabled":true}), &catalog)?;
        }
        Some("read") => { assert_eq!(config::shortcuts()["favorites"], json!([key])); }
        _ => return Err("invalid phase".into()),
    }
    println!("phase_ok");
    Ok(())
}
