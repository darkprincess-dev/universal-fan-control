use glob::glob;
use std::fs;

pub fn scan_system() -> anyhow::Result<()> {
    println!(
        "{:<20} | {:<20} | HWMON PATH",
        "DEVICE NAME", "ADDRESS (UID Prefix)"
    );
    println!("{}", "-".repeat(80));

    let base_dir = "/sys/class/hwmon";
    let mut entries: Vec<_> = glob(&format!("{}/hwmon*", base_dir))?
        .filter_map(Result::ok)
        .collect();
    entries.sort();

    for path in entries {
        // 1. Read name
        let name = fs::read_to_string(path.join("name"))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        // 2. Extract address
        let mut address = "Virtual/Unknown".to_string();
        let device_link = path.join("device");
        if device_link.exists()
            && let Ok(real_path) = fs::read_link(&device_link) {
                // Canonicalize if relative
                let full_real_path = if real_path.is_relative() {
                    path.join("device").canonicalize().unwrap_or(real_path)
                } else {
                    real_path
                };

                if let Some(os_name) = full_real_path.file_name() {
                    address = os_name.to_string_lossy().to_string();
                    if address.starts_with("0000:") {
                        address = address[5..].to_string();
                    }
                }
            }

        println!("{:<20} | {:<20} | {}", name, address, path.display());

        // 3. Sensor/PWM preview
        let mut pwm_files: Vec<String> = Vec::new();
        let mut sensor_files: Vec<String> = Vec::new();

        if let Ok(read_dir) = fs::read_dir(&path) {
            for entry in read_dir.filter_map(Result::ok) {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with("pwm")
                    && file_name
                        .chars()
                        .nth(3)
                        .is_some_and(|c| c.is_ascii_digit())
                {
                    pwm_files.push(file_name);
                } else if file_name.ends_with("_input") && file_name.contains("temp") {
                    sensor_files.push(file_name);
                }
            }
        }

        pwm_files.sort();
        sensor_files.sort();

        if !pwm_files.is_empty() {
            println!("   ↳ Controls: {}", pwm_files.join(", "));
        }
        if !sensor_files.is_empty() {
            let limit = sensor_files.len().min(3);
            println!("   ↳ Sensors:  {} ...", sensor_files[..limit].join(", "));
        }
        println!("{}", "-".repeat(80));
    }

    Ok(())
}
