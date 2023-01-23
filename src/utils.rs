/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use std::net;

use ifcfg::IfCfg;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct IfaceInfo {
    pub(crate) name: String,
    pub(crate) ip: net::IpAddr,
    pub(crate) index: u32,
}

impl TryFrom<IfCfg> for IfaceInfo {
    type Error = &'static str;

    fn try_from(value: IfCfg) -> Result<Self, Self::Error> {
        if value.addresses.is_empty() {
            return Err("Address is unknown");
        }

        match value
            .addresses
            .iter()
            .find(|addr| matches!(addr.address, Some(net::SocketAddr::V4(_))))
        {
            Some(iface_addr) => Ok(IfaceInfo {
                name: value.name,
                ip: iface_addr.address.unwrap().ip(),
                index: 0,
            }),
            None => Err("No supported IP address found"),
        }
    }
}
