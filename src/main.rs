/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use network_interface::{self, NetworkInterface, NetworkInterfaceConfig};

use utils::IfaceInfo;

mod utils;

fn main() {
    let mut stored_iface: Option<IfaceInfo> = None;

    loop {
        let net = NetworkInterface::show();

        for iface in net.unwrap().iter().filter(|it_iface| {
            vpn_iface_check(it_iface)
                && matches!(it_iface.addr, Some(network_interface::Addr::V4(_)))
        }) {
            if let Ok(ser_iface) = IfaceInfo::try_from(iface.to_owned()) {
                if stored_iface.is_none() || stored_iface.as_ref().unwrap() != &ser_iface {
                    stored_iface = Some(ser_iface);
                    dbg!(&stored_iface);
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}

#[cfg(unix)]
fn vpn_iface_check(iface: &NetworkInterface) -> bool {
    iface.name.starts_with("tun")
}

#[cfg(windows)]
fn vpn_iface_check(iface: &NetworkInterface) -> bool {
    false
}
