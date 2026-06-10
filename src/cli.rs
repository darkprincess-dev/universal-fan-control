use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fan-control")]
#[command(about = "Linux Universal PWM Fan Controller", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan the system for hwmon devices
    Scan,
    /// Run the fan controller
    Run(RunArgs),
}

#[derive(clap::Args)]
pub struct RunArgs {
    /// Physical UID of the temperature device(s). format: {address}@{type}@{index or _attribute}@{_attribute}
    #[arg(short = 't', long = "temp", num_args = 1..)]
    pub temp: Vec<String>,

    /// Temperature mode: avg, min, or max
    #[arg(long = "temp_mode", alias = "tm", default_value = "avg")]
    pub temp_mode: String,

    /// Physical UID of the fan device(s). format: {address}@{type}@{index or _attribute}@{_attribute}
    #[arg(short = 'f', long = "fan", num_args = 1..)]
    pub fan: Vec<String>,

    /// Fan activation properties attribute (e.g., _enable)
    #[arg(long = "fan_mode", alias = "fm")]
    pub fan_mode_attr: String,

    /// Desired temperature and fan speed. format: {temp}:{percent}
    #[arg(long = "fan_config", alias = "fc", num_args = 1..)]
    pub fan_config: Vec<String>,
}
