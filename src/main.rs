/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use log::{debug, warn};
use network_interface::{self, NetworkInterface, NetworkInterfaceConfig};
use serde::Deserialize;

use utils::IfaceInfo;

mod utils;

const REPORT_URL_VAR: &str = "IPREPORT_ADDR";
const TOKEN_ENV_VAR: &str = "IPREPORT_APP_TOKEN";
const CONFIG_JSON_FILE: &str = "config.json";
const REPORT_CERT_FILE: &str = "cert.pem";

#[derive(Debug, PartialEq, Deserialize)]
struct TrackerConfig {
    token: String,
    #[serde(rename(deserialize = "reportUrl"))]
    report_url: String,
}

impl TrackerConfig {
    fn from_env() -> Option<Self> {
        let report_url = std::env::var(REPORT_URL_VAR);
        let token = std::env::var(TOKEN_ENV_VAR);

        if report_url.is_err() || token.is_err() {
            return None;
        }

        let report_url = report_url.unwrap();
        let token = token.unwrap();

        Some(Self { token, report_url })
    }
}

fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    let mut stored_iface: Option<IfaceInfo> = None;
    let config = load_config();

    if config.is_none() {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }

    let config = config.unwrap();

    loop {
        let net = NetworkInterface::show();

        for iface in net.unwrap().iter().filter(|it_iface| {
            vpn_iface_check(it_iface)
                && matches!(it_iface.addr, Some(network_interface::Addr::V4(_)))
        }) {
            if let Ok(ser_iface) = IfaceInfo::try_from(iface.to_owned()) {
                if stored_iface.is_none() || stored_iface.as_ref().unwrap() != &ser_iface {
                    match send_report(&ser_iface, &config) {
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

fn load_config() -> Option<TrackerConfig> {
    let config = std::fs::File::open(CONFIG_JSON_FILE);

    if let Ok(config) = config {
        if let Ok(config) = serde_json::from_reader(&config) {
            return Some(config);
        }
    }

    TrackerConfig::from_env()
}

fn send_report(iface: &IfaceInfo, config: &TrackerConfig) -> reqwest::Result<()> {
    let data = iface.ip.to_string();
    let headers = prepare_headers(config.token.clone());
    let url = config.report_url.clone();
    let cert = std::fs::read(REPORT_CERT_FILE).expect("Could not find valid certificate");
    let cert = reqwest::Certificate::from_pem(cert.as_slice()).unwrap();
    let client = reqwest::blocking::Client::builder()
        .tls_sni(false)
        .add_root_certificate(cert)
        .build()
        .unwrap();

    client
        .post(url)
        .headers(headers)
        .body(data)
        .timeout(core::time::Duration::from_secs(10))
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
fn vpn_iface_check(iface: &NetworkInterface) -> bool {
    iface.name.starts_with("tun")
}

#[cfg(windows)]
fn vpn_iface_check(iface: &NetworkInterface) -> bool {
    false
}

#[cfg(test)]
mod config_tests {
    use crate::{load_config, CONFIG_JSON_FILE, REPORT_URL_VAR, TOKEN_ENV_VAR};

    use std::env;

    const TEST_TOKEN: &str = "some_env_token";
    const TEST_URL: &str = "https://some_url/";

    #[test]
    fn test_load_config_no_available() {
        let config = load_config();
        assert_eq!(config, None);
    }

    #[test]
    fn test_load_config_env() {
        fn setup(exp_token: &str, exp_url: &str) {
            env::set_var(TOKEN_ENV_VAR, exp_token);
            env::set_var(REPORT_URL_VAR, exp_url);
        }

        fn teardown() {
            env::remove_var(TOKEN_ENV_VAR);
            env::remove_var(REPORT_URL_VAR);
        }

        setup(TEST_TOKEN, TEST_URL);
        let config = load_config().unwrap();

        assert_eq!(config.token, TEST_TOKEN);
        assert_eq!(config.report_url, TEST_URL);
        teardown();
    }

    #[test]
    fn test_load_token_file() {
        fn setup(exp_token: &str, exp_url: &str) {
            let data = serde_json::json![{
                "token": exp_token,
                "reportUrl": exp_url,
            }];
            let f = std::fs::File::create(CONFIG_JSON_FILE).unwrap();
            serde_json::ser::to_writer_pretty(f, &data).unwrap();
        }

        fn teardown() {
            std::fs::remove_file(CONFIG_JSON_FILE).expect("Failed to remove test config.json");
        }

        setup(TEST_TOKEN, TEST_URL);
        let config = load_config().unwrap();

        assert_eq!(config.token, TEST_TOKEN);
        assert_eq!(config.report_url, TEST_URL);
        teardown();
    }
}
