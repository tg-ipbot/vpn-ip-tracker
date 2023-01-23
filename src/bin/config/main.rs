use std::io;

use clap::{Parser, Subcommand};
use confy::ConfyError;
use thiserror::Error;

#[cfg(unix)]
use config_linux::{config_setup, install_service, uninstall_service};
#[cfg(windows)]
use config_win::{config_setup, install_service, uninstall_service};
use vpn_ip_tracker::{TrackerConfig, DEFAULT_REPORT_URL};

#[cfg(unix)]
mod config_linux;

#[cfg(windows)]
mod config_win;

#[derive(Debug, Parser)]
#[command(author, about = "VPN IP Tracker configuration", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Install VPN IP Tracker service")]
    Install {
        #[arg(short, long, help = "Application token")]
        token: String,
        #[arg(short, long, help = "URL to send reports with IP address info")]
        report_url: Option<String>,
    },
    #[command(about = "Uninstall VPN IP Tracker service")]
    Uninstall,
}

#[derive(Debug, Error)]
enum ConfigError {
    #[error("path error")]
    Path(#[from] io::Error),
    #[error("configuration file error")]
    ConfigFile(#[from] ConfyError),
    #[cfg(target_os = "windows")]
    #[error("windows service error")]
    Service(#[from] windows_service::Error),
}

fn main() -> Result<(), ConfigError> {
    let args = Cli::parse();

    match args.command {
        Commands::Install { token, report_url } => {
            config_setup(TrackerConfig::new(
                token,
                report_url.unwrap_or_else(|| DEFAULT_REPORT_URL.into()),
            ))?;
            install_service()?;
        }
        Commands::Uninstall => {
            uninstall_service()?;
        }
    }

    Ok(())
}
