use crate::cli::RunArgs;
use crate::hwmon::uid_to_path;
use anyhow::{Result, anyhow};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct FanState {
    path: PathBuf,
    original_mode: String,
}

pub fn run_controller(args: RunArgs) -> Result<()> {
    // 1. Resolve paths
    let temp_paths: Vec<PathBuf> = args
        .temp
        .iter()
        .map(|t| uid_to_path(t))
        .collect::<Result<Vec<_>>>()?;

    let fan_paths: Vec<PathBuf> = args
        .fan
        .iter()
        .map(|f| uid_to_path(f))
        .collect::<Result<Vec<_>>>()?;

    // 2. Prepare fan modes and backup original state
    let mut fan_states = Vec::new();
    for f_path in &fan_paths {
        let mut mode_path = f_path.clone();
        // The python code says: fan_mode_path = f_path + args.fan_mode
        // It assumes f_path is like /.../pwm1 and fan_mode is like _enable
        let file_name = f_path
            .file_name()
            .ok_or_else(|| anyhow!("Invalid fan path"))?
            .to_string_lossy();
        let mode_file_name = format!("{}{}", file_name, args.fan_mode_attr);
        mode_path.set_file_name(mode_file_name);

        if !mode_path.exists() {
            return Err(anyhow!(
                "Fan mode path does not exist: {}",
                mode_path.display()
            ));
        }

        let original_mode = fs::read_to_string(&mode_path)?.trim().to_string();
        fan_states.push(FanState {
            path: mode_path.clone(),
            original_mode,
        });

        // Set to manual mode (usually "1")
        fs::write(&mode_path, "1")?;
    }

    let fan_states = Arc::new(Mutex::new(fan_states));
    let fan_states_clone = Arc::clone(&fan_states);

    // 3. Setup Signal Handler
    ctrlc::set_handler(move || {
        println!("\nShutting down, restoring fan modes...");
        let states = fan_states_clone.lock().unwrap();
        for state in states.iter() {
            let _ = fs::write(&state.path, &state.original_mode);
        }
        std::process::exit(0);
    })?;

    // 4. Parse fan config
    let mut fan_config = Vec::new();
    for cfg in args.fan_config {
        let parts: Vec<&str> = cfg.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid fan config: {}", cfg));
        }
        let temp: f64 = parts[0].parse()?;
        let percent: f64 = parts[1].parse()?;
        let pwm = (percent.clamp(0.0, 100.0) * 255.0 / 100.0) as i32;
        fan_config.push((temp, pwm));
    }
    fan_config.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // 5. Run loop
    println!("Fan controller started. Press Ctrl+C to stop.");
    let mut temp_buff = vec![0.0; temp_paths.len()];

    loop {
        for (i, p) in temp_paths.iter().enumerate() {
            let content = fs::read_to_string(p)?;
            temp_buff[i] = content.trim().parse::<f64>()? / 1000.0;
        }

        let current_temp = match args.temp_mode.as_str() {
            "avg" => temp_buff.iter().sum::<f64>() / temp_buff.len() as f64,
            "min" => *temp_buff
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap(),
            "max" => *temp_buff
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap(),
            _ => return Err(anyhow!("Unknown temp mode: {}", args.temp_mode)),
        };

        let pwm = get_linear_pwm(current_temp, &fan_config);

        for f_path in &fan_paths {
            fs::write(f_path, pwm.to_string())?;
        }

        thread::sleep(Duration::from_secs(2));
    }
}

fn get_linear_pwm(current_temp: f64, config: &[(f64, i32)]) -> i32 {
    if config.is_empty() {
        return 0;
    }
    if current_temp <= config[0].0 {
        return config[0].1;
    }
    if current_temp >= config.last().unwrap().0 {
        return config.last().unwrap().1;
    }

    for i in 0..config.len() - 1 {
        let (t1, p1) = config[i];
        let (t2, p2) = config[i + 1];
        if current_temp >= t1 && current_temp <= t2 {
            let ratio = (current_temp - t1) / (t2 - t1);
            let interpolated = p1 as f64 + (p2 - p1) as f64 * ratio;
            return interpolated as i32;
        }
    }

    config.last().unwrap().1
}
