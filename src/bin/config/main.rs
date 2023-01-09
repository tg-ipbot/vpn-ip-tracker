use clap::Parser;
use confy::ConfyError;

use vpn_ip_tracker::{TrackerConfig, APP_NAME, DEFAULT_REPORT_URL};

#[cfg(target_os = "linux")]
mod config_linux;
#[cfg(target_os = "linux")]
use config_linux::install_service;

#[cfg(target_os = "windows")]
mod config_win;
#[cfg(target_os = "windows")]
use config_win::install_service;

#[derive(Debug, Parser)]
#[command(author, about = "VPN IP Tracker configuration", version)]
struct Cli {
    #[arg(short, long, help = "Application token")]
    token: String,
    #[arg(short, long, help = "URL to send reports with IP address info")]
    report_url: Option<String>,
}

fn main() -> Result<(), ConfyError> {
    let args = Cli::parse();

    confy::store(
        APP_NAME,
        None,
        TrackerConfig::new(
            args.token,
            args.report_url.unwrap_or_else(|| DEFAULT_REPORT_URL.into()),
        ),
    )?;
    install_service().map_err(ConfyError::GeneralLoadError)
}
