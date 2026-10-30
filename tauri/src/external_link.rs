use url::Url;

pub fn validate(input: &str) -> Result<Url, &'static str> {
    if input.len() > 8192 || input.chars().any(char::is_control) {
        return Err("invalid URL");
    }
    let url = Url::parse(input).map_err(|_| "invalid URL")?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none()
        || !url.username().is_empty() || url.password().is_some() {
        return Err("unsupported URL");
    }
    Ok(url)
}

#[cfg(windows)]
pub fn open(url: &Url) -> Result<(), &'static str> {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    let target: Vec<u16> = OsStr::new(url.as_str()).encode_wide().chain(Some(0)).collect();
    let operation: Vec<u16> = OsStr::new("open").encode_wide().chain(Some(0)).collect();
    let result = unsafe { ShellExecuteW(std::ptr::null_mut(), operation.as_ptr(), target.as_ptr(), std::ptr::null(), std::ptr::null(), 1) };
    if result as isize <= 32 { Err("cannot open URL") } else { Ok(()) }
}

#[cfg(not(windows))]
pub fn open(url: &Url) -> Result<(), &'static str> {
    #[cfg(target_os = "macos")]
    let program = "open";
    #[cfg(not(target_os = "macos"))]
    let program = "xdg-open";
    std::process::Command::new(program).arg(url.as_str()).spawn().map(|_| ()).map_err(|_| "cannot open URL")
}

#[cfg(test)]
mod tests {
    use super::validate;
    #[test]
    fn accepts_only_safe_web_links() {
        for link in ["https://example.com/a?q=1#part", "http://127.0.0.1:8080/"] { assert!(validate(link).is_ok()); }
        for link in ["javascript:alert(1)", "file:///C:/test", "https://user:pass@example.com", "https://example.com/\nfoo", "https://"] { assert!(validate(link).is_err()); }
    }
}
