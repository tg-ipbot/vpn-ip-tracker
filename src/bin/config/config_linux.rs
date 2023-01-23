use std::io::{BufRead, BufReader, Error, ErrorKind, Write};

use directories::BaseDirs;

use vpn_ip_tracker::{TrackerConfig, APP_NAME};

use crate::ConfigError;

type Result<T> = core::result::Result<T, ConfigError>;

pub(super) fn config_setup(config: TrackerConfig) -> Result<()> {
    confy::store(APP_NAME, None, config)?;

    Ok(())
}

pub(super) fn install_service() -> Result<()> {
    const TEMPLATE_SERVICE_FILENAME: &str = "vpn-ip-tracker.service.template";
    let executable_path = std::env::current_exe()?;
    let parent_dir = executable_path.parent().unwrap();
    let template_file = std::fs::File::open(TEMPLATE_SERVICE_FILENAME)?;
    let service_dir = BaseDirs::new()
        .unwrap()
        .config_dir()
        .join("systemd")
        .join("user");
    std::fs::create_dir_all(&service_dir)?;

    let reader = BufReader::new(template_file);
    let mut service_file = std::fs::File::create(service_dir.join("vpn-ip-tracker.service"))?;

    for line in reader.lines().flatten() {
        let sub_start = line.find('@');

        if let Some(sub_start) = sub_start {
            let sub_end = line.rfind('@').unwrap();
            let mut dir_str = String::from(&line[..sub_start]);

            dir_str += parent_dir.to_str().unwrap();
            dir_str += &line[sub_end + 1..];
            writeln!(service_file, "{dir_str}")
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        } else {
            writeln!(service_file, "{line}").map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        }
    }

    Ok(())
}

pub(super) fn uninstall_service() -> Result<()> {
    todo!()
}
