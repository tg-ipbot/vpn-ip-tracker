/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use clap::Parser;
use ifcfg::IfCfg;
use log::{debug, warn};

use utils::IfaceInfo;
use vpn_ip_tracker::TrackerConfig;

mod utils;

#[derive(Parser)]
#[command(author, version)]
struct Cli {
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Debug)]
enum AppError {
    ConfigInvalid,
}

fn main() -> Result<(), AppError> {
    let args = Cli::parse();

    if args.verbose {
        env_logger::init();
    }

    let mut stored_iface: Option<IfaceInfo> = None;
    let config = TrackerConfig::load();

    if config.is_none() {
        return Err(AppError::ConfigInvalid);
    }

    let config = config.unwrap();
    let client = reqwest::blocking::Client::builder()
        .timeout(core::time::Duration::from_secs(10))
        .build()
        .unwrap();

    loop {
        let net = IfCfg::get().expect("Unable to get network interface info");

        for iface in net
            .into_iter()
            .filter(|it_iface| vpn_iface_name_check(it_iface) && vpn_iface_ipv4_check(it_iface))
        {
            if let Ok(ser_iface) = IfaceInfo::try_from(iface) {
                if stored_iface.is_none() || stored_iface.as_ref().unwrap() != &ser_iface {
                    match send_report(client.clone(), &ser_iface, &config) {
                        Ok(_) => {
                            debug!("Successfully report");
                            stored_iface = Some(ser_iface);
                            debug!("{:?}", &stored_iface);
                        }
                        Err(e) => warn!("Failed to send report: {}", e),
                    }

                    break;
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}

fn send_report(
    client: reqwest::blocking::Client,
    iface: &IfaceInfo,
    config: &TrackerConfig,
) -> reqwest::Result<()> {
    let data = iface.ip.to_string();
    let headers = prepare_headers(config.token.clone());
    let url = config.report_url.clone();

    client
        .post(url)
        .headers(headers)
        .body(data)
        .send()?
        .error_for_status()?;

    Ok(())
}

fn prepare_headers(token: String) -> reqwest::header::HeaderMap {
    let mut header = reqwest::header::HeaderMap::new();
    let mut token = reqwest::header::HeaderValue::from_str(&token).unwrap();

    token.set_sensitive(true);
    header.insert("Credential", token);

    header
}

#[cfg(unix)]
fn vpn_iface_name_check(iface: &IfCfg) -> bool {
    iface.name.starts_with("tun")
}

#[cfg(windows)]
fn vpn_iface_name_check(iface: &IfCfg) -> bool {
    iface.name.contains("OpenVPN TAP")
}

fn vpn_iface_ipv4_check(iface: &IfCfg) -> bool {
    iface
        .addresses
        .iter()
        .any(|addr| matches!(addr.address_family, ifcfg::AddressFamily::IPv4))
}
