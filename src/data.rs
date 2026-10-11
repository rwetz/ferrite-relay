use std::path::PathBuf;
pub struct Playing {
    pub station: String,
    pub track: String,
}
pub fn parse_playing(text: &str, now: u64) -> Option<Playing> {
    let data: serde_json::Value = serde_json::from_str(text).ok()?;
    let time = data["time"].as_u64()?;
    if data["version"].as_u64() != Some(1)
        || data["playing"].as_bool() != Some(true)
        || time > now
        || now - time > 10
    {
        return None;
    }
    Some(Playing {
        station: data["station"].as_str()?.chars().take(100).collect(),
        track: data["track"].as_str()?.chars().take(180).collect(),
    })
}
pub fn playing() -> Option<Playing> {
    let base = std::env::var_os(if cfg!(windows) { "APPDATA" } else { "HOME" })?;
    let path = PathBuf::from(base).join("ferrite/wireless-now-playing.json");
    if std::fs::metadata(&path).ok()?.len() > 8192 {
        return None;
    }
    let text = std::fs::read_to_string(path).ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    parse_playing(&text, now)
}
pub struct Cpu {
    previous: Option<(u64, u64)>,
    pub usage: Option<f32>,
}
impl Cpu {
    pub fn new() -> Self {
        Self {
            previous: None,
            usage: None,
        }
    }
    pub fn sample(&mut self) {
        if let Some((idle, total)) = times() {
            self.usage = self.previous.and_then(|(old_idle, old_total)| {
                let elapsed = total.checked_sub(old_total)?;
                let idle = idle.checked_sub(old_idle)?;
                (elapsed > 0).then(|| 100. * (elapsed.saturating_sub(idle)) as f32 / elapsed as f32)
            });
            self.previous = Some((idle, total));
        }
    }
}
#[cfg(windows)]
fn times() -> Option<(u64, u64)> {
    #[repr(C)]
    #[derive(Default)]
    struct FileTime {
        low: u32,
        high: u32,
    }
    impl FileTime {
        fn value(&self) -> u64 {
            u64::from(self.low) | (u64::from(self.high) << 32)
        }
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetSystemTimes(idle: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime) -> i32;
    }
    let (mut idle, mut kernel, mut user) = (
        FileTime::default(),
        FileTime::default(),
        FileTime::default(),
    );
    if unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) } == 0 {
        return None;
    }
    Some((idle.value(), kernel.value() + user.value()))
}
#[cfg(not(windows))]
fn times() -> Option<(u64, u64)> {
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stopped_stale_future_and_unknown_snapshots_never_claim_to_be_live() {
        let valid = r#"{"version":1,"time":100,"playing":true,"station":"Jazz","track":"Track"}"#;
        assert!(parse_playing(valid, 105).is_some());
        assert!(parse_playing(valid, 111).is_none());
        assert!(parse_playing(valid, 99).is_none());
        assert!(parse_playing(&valid.replace("true", "false"), 105).is_none());
        assert!(parse_playing(&valid.replace("\"version\":1", "\"version\":2"), 105).is_none());
    }
}
