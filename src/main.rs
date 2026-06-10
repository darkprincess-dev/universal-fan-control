mod cli;
mod controller;
mod hwmon;
mod scanner;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan => {
            scanner::scan_system()?;
        }
        Commands::Run(args) => {
            // Check for root privileges on Linux
            #[cfg(target_os = "linux")]
            {
                if unsafe { libc::geteuid() } != 0 {
                    anyhow::bail!("Error: This command requires root privileges (sudo).");
                }
            }

            controller::run_controller(args)?;
        }
    }

    Ok(())
}
