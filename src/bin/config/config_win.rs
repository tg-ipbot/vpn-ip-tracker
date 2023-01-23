/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use std::{ffi::OsString, thread, time::Duration};

use vpn_ip_tracker::{TrackerConfig, APP_NAME, APP_SERVICE_NAME};
use windows_service::{
    service::{
        ServiceAccess, ServiceAction, ServiceActionType, ServiceErrorControl,
        ServiceFailureActions, ServiceFailureResetPeriod, ServiceInfo, ServiceStartType,
        ServiceState, ServiceType,
    },
    service_manager::{ServiceManager, ServiceManagerAccess},
};

use crate::ConfigError;

type Result<T> = core::result::Result<T, ConfigError>;

pub(super) fn config_setup(config: TrackerConfig) -> Result<()> {
    let binding = std::env::current_exe().map_err(ConfigError::Path)?;
    let current_exe_path = binding.with_file_name(APP_NAME);

    confy::store_path(current_exe_path, config)?;

    Ok(())
}

pub(super) fn install_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;
    let service_binary_path = std::env::current_exe()
        .unwrap()
        .with_file_name(format!("{APP_SERVICE_NAME}.exe"));
    let openvpn_service_dep =
        windows_service::service::ServiceDependency::Service(OsString::from("OpenVPNService"));
    let service_info = ServiceInfo {
        name: OsString::from(APP_NAME),
        display_name: OsString::from("VpnIpTracking"),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::OnDemand,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: vec![],
        dependencies: vec![openvpn_service_dep],
        account_name: None, // run as System
        account_password: None,
    };
    let service = service_manager.create_service(
        &service_info,
        ServiceAccess::CHANGE_CONFIG | ServiceAccess::START,
    )?;

    let actions = vec![ServiceAction {
        action_type: ServiceActionType::Restart,
        delay: Duration::from_secs(5),
    }];

    let reset_period = Duration::from_secs(86400 * 2);
    let failure_actions = ServiceFailureActions {
        reset_period: ServiceFailureResetPeriod::After(reset_period),
        reboot_msg: None,
        command: None,
        actions: Some(actions),
    };

    service.update_failure_actions(failure_actions)?;
    service.set_failure_actions_on_non_crash_failures(true)?;
    service.set_description("VPN IP Tracking Service that reports your VPN IP address")?;

    Ok(())
}

pub(super) fn uninstall_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;
    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(OsString::from(APP_NAME), service_access)?;
    let service_status = service.query_status()?;

    if service_status.current_state != ServiceState::Stopped {
        service.stop()?;
        // Wait for service to stop
        thread::sleep(Duration::from_secs(1));
    }

    service.delete()?;
    Ok(())
}
