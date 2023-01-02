/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use log::{debug, warn};
use network_interface::{self, NetworkInterface, NetworkInterfaceConfig};
use std::io::Read;

use utils::IfaceInfo;

mod utils;

const REPORT_URL_VAR: &str = "IPREPORT_ADDR";
const TOKEN_ENV_VAR: &str = "IPREPORT_APP_TOKEN";
const CONFIG_JSON_FILE: &str = "config.json";

fn main() {
    env_logger::init();
    let mut stored_iface: Option<IfaceInfo> = None;

    loop {
        let net = NetworkInterface::show();

        for iface in net.unwrap().iter().filter(|it_iface| {
            vpn_iface_check(it_iface)
                && matches!(it_iface.addr, Some(network_interface::Addr::V4(_)))
        }) {
            if let Ok(ser_iface) = IfaceInfo::try_from(iface.to_owned()) {
                if stored_iface.is_none() || stored_iface.as_ref().unwrap() != &ser_iface {
                    match send_report(&ser_iface) {
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

fn load_token() -> Option<String> {
    if let Ok(token) = std::env::var(TOKEN_ENV_VAR) {
        return Some(token);
    }

    match std::fs::File::open(CONFIG_JSON_FILE) {
        Ok(mut config) => {
            let mut buffer = String::new();

            if config.read_to_string(&mut buffer).is_ok() {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&buffer) {
                    if let Some(token) = value["token"].as_str() {
                        return Some(token.into());
                    }
                }
            }
        }
        _ => warn!("config.json file has not been found"),
    }

    None
}

fn send_report(iface: &IfaceInfo) -> reqwest::Result<()> {
    let data = iface.ip.to_string();
    let token = load_token();

    if token.is_none() {
        panic!("Please specify token either in IPREPORT_APP_TOKEN or in the config.json file");
    }

    let headers = prepare_headers(token.unwrap());
    let url =
        std::env::var(REPORT_URL_VAR).expect("Please specify IPREPORT_ADDR environment variable");
    let client = reqwest::blocking::Client::new();

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

    header.insert("Credential", token.parse().unwrap());

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
mod token_tests {
    use crate::{load_token, CONFIG_JSON_FILE, TOKEN_ENV_VAR};

    use std::env;

    #[test]
    fn test_load_token_no_available() {
        let token = load_token();
        assert_eq!(token, None);
    }

    #[test]
    fn test_load_token_env() {
        const TEST_TOKEN: &str = "some_env_token";

        fn setup(exp_token: &str) {
            env::set_var(TOKEN_ENV_VAR, exp_token);
        }

        fn teardown() {
            env::remove_var(TOKEN_ENV_VAR);
        }

        setup(TEST_TOKEN);
        let token = env::var(TOKEN_ENV_VAR).expect("Failed to get token");
        assert_eq!(token, TEST_TOKEN);
        teardown();
    }

    #[test]
    fn test_load_token_file() {
        const TEST_TOKEN: &str = "some_file_token";

        fn setup(exp_token: &str) {
            let data = serde_json::json![{
                "token": exp_token,
            }];
            let f = std::fs::File::create(CONFIG_JSON_FILE).unwrap();
            serde_json::ser::to_writer_pretty(f, &data).unwrap();
        }

        fn teardown() {
            std::fs::remove_file(CONFIG_JSON_FILE).expect("Failed to remove test config.json");
        }

        setup(TEST_TOKEN);
        let token = load_token();
        assert!(token.is_some());
        assert_eq!(token.unwrap(), TEST_TOKEN);
        teardown();
    }
}
