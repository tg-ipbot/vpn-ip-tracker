/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use std::net;

use network_interface::NetworkInterface;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct IfaceInfo {
    pub(crate) name: String,
    pub(crate) ip: net::IpAddr,
    pub(crate) index: u32,
}

impl TryFrom<NetworkInterface> for IfaceInfo {
    type Error = &'static str;

    fn try_from(value: NetworkInterface) -> Result<Self, Self::Error> {
        if value.addr.is_none() {
            return Err("Address is unknown");
        }

        Ok(IfaceInfo {
            name: value.name,
            ip: value.addr.unwrap().ip(),
            index: value.index,
        })
    }
}
