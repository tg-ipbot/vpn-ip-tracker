/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use confy::ConfyError;
use serde::{Deserialize, Serialize};

/// Application name that is used for configuration stuff
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
#[cfg(windows)]
pub const APP_SERVICE_NAME: &str = "vpn-ip-tracker-svc";
/// Default report service URL
pub const DEFAULT_REPORT_URL: &str = env!("VPN_IP_TRACKER_REPORT_URL");
/// Environment variable name that provides report URL to send VPN IP address reports
const REPORT_URL_VAR: &str = "IPREPORT_ADDR";
/// Environment variable name that provides application token that IpBot provided
const TOKEN_ENV_VAR: &str = "IPREPORT_APP_TOKEN";

/// Tracker configuration loaded from the config file
#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrackerConfig {
    /// Application token
    pub token: String,
    /// Report URL
    pub report_url: String,
}

impl TrackerConfig {
    /// Create new tracker configuration
    /// Arguments:
    /// - `token` - application token string
    /// - `report_url` - report service URL
    pub fn new(token: String, report_url: String) -> Self {
        Self { token, report_url }
    }

    /// Try to load tracker configuration
    /// 1. tries to get configuration from OS specific user configuration directory
    /// 2. if the configuration is not available, try to load configuration from the environment
    /// variables (see [`REPORT_URL_VAR`](REPORT_URL_VAR) and [`TOKEN_ENV_VAR`](TOKEN_ENV_VAR))
    pub fn load() -> Option<Self> {
        if let Ok(config) = Self::load_config() {
            if config != TrackerConfig::default() {
                return Some(config);
            }
        }

        Self::from_env()
    }

    #[cfg(windows)]
    fn load_config() -> Result<Self, ConfyError> {
        let current_dir_config = std::env::current_exe().unwrap().with_file_name(APP_NAME);

        confy::load_path(current_dir_config)
    }

    #[cfg(unix)]
    fn load_config() -> Result<Self, ConfyError> {
        confy::load(APP_NAME, None)
    }

    /// Try to load tracker configuration from the environment variables
    /// (see [`REPORT_URL_VAR`](REPORT_URL_VAR) and [`TOKEN_ENV_VAR`](TOKEN_ENV_VAR))
    pub fn from_env() -> Option<Self> {
        let report_url = std::env::var(REPORT_URL_VAR);
        let token = std::env::var(TOKEN_ENV_VAR);

        if report_url.is_err() || token.is_err() {
            return None;
        }

        let report_url = report_url.unwrap();
        let token = token.unwrap();

        Some(Self::new(token, report_url))
    }
}

#[cfg(test)]
mod config_tests {
    use std::env;

    use crate::{TrackerConfig, APP_NAME, REPORT_URL_VAR, TOKEN_ENV_VAR};

    const TEST_TOKEN: &str = "some_env_token";
    const TEST_URL: &str = "https://some_url/";

    fn load_config() -> Option<TrackerConfig> {
        TrackerConfig::load()
    }

    #[test]
    fn test_config_new() {
        let config = TrackerConfig::new(TEST_TOKEN.into(), TEST_URL.into());

        assert_eq!(config.token, TEST_TOKEN);
        assert_eq!(config.report_url, TEST_URL);
    }

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
            let _ = confy::store(
                APP_NAME,
                None,
                TrackerConfig {
                    token: exp_token.to_string(),
                    report_url: exp_url.to_string(),
                },
            );
        }

        fn teardown() {
            std::fs::remove_file(confy::get_configuration_file_path(APP_NAME, None).unwrap())
                .expect("Failed to remove configuration");
        }

        setup(TEST_TOKEN, TEST_URL);
        let config = load_config().unwrap();

        assert_eq!(config.token, TEST_TOKEN);
        assert_eq!(config.report_url, TEST_URL);
        teardown();
    }
}
