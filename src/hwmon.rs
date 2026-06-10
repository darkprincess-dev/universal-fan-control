use anyhow::{Result, anyhow};
use glob::glob;
use std::fs;
use std::path::PathBuf;

pub fn uid_to_path(uid: &str) -> Result<PathBuf> {
    let parts: Vec<&str> = uid.trim().split('@').collect();
    let (addr, type_, idx, attr) = match parts.len() {
        2 => (parts[0], parts[1], "", ""),
        3 => (parts[0], parts[1], parts[2], ""),
        4 => (parts[0], parts[1], parts[2], parts[3]),
        _ => return Err(anyhow!("Invalid UID format: {}", uid)),
    };

    let addr_lower = addr.to_lowercase();
    let mut hwmon_path = None;

    for entry in glob("/sys/class/hwmon/hwmon*")?.filter_map(Result::ok) {
        let device_link = entry.join("device");
        if device_link.is_symlink()
            && let Ok(real_path) = fs::read_link(&device_link) {
                // Canonicalize to get full path if it's relative
                let full_real_path = entry.join("device").canonicalize().unwrap_or(real_path);
                if full_real_path
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(&addr_lower)
                {
                    hwmon_path = Some(entry);
                    break;
                }
            }
    }

    let hwmon = hwmon_path
        .ok_or_else(|| anyhow!("Device with address '{}' not found for UID '{}'", addr, uid))?;
    let file_name = format!("{}{}{}", type_, idx, attr);
    let final_path = hwmon.join(file_name);

    if !final_path.exists() {
        return Err(anyhow!("Path does not exist: {}", final_path.display()));
    }

    Ok(final_path)
}
