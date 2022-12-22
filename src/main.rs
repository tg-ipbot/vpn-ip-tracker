/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use std::fmt::Formatter;
use std::io::Seek;
use std::mem::size_of;

use network_interface::{self, NetworkInterface, NetworkInterfaceConfig};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tempfile::NamedTempFile;

struct NetworkIfaceWrapper(NetworkInterface);

impl PartialEq for NetworkIfaceWrapper {
    fn eq(&self, other: &Self) -> bool {
        let addr_equal = match (&self.0.addr, &other.0.addr) {
            (Some(network_interface::Addr::V4(a)), Some(network_interface::Addr::V4(b))) => {
                a.ip == b.ip
            }
            _ => false,
        };

        self.0.name == other.0.name && addr_equal && self.0.index == other.0.index
    }
}

impl Serialize for NetworkIfaceWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("NetworkInterface", size_of::<Self>())?;

        state.serialize_field("name", &self.0.name)?;
        state.serialize_field("index", &self.0.index)?;
        state.serialize_field("addr", &self.0.addr.unwrap().ip())?;
        state.serialize_field("mac", &self.0.mac_addr)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for NetworkIfaceWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Name,
            Index,
            Addr,
            Mac,
        }

        struct NetworkIfaceVisitor;

        impl<'de> Visitor<'de> for NetworkIfaceVisitor {
            type Value = NetworkIfaceWrapper;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct NetworkIfaceWrapper")
            }

            fn visit_seq<V>(self, mut _seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                Ok(NetworkIfaceWrapper(NetworkInterface {
                    name: "".to_string(),
                    index: 0,
                    addr: None,
                    mac_addr: None,
                }))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut name = None;
                let mut index = None;
                let mut addr = None;
                let mut _mac_addr = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            name = Some(map.next_value::<String>()?);
                        }
                        Field::Index => {
                            index = Some(map.next_value()?);
                        }
                        Field::Addr => {
                            addr = Some(map.next_value::<std::net::Ipv4Addr>()?);
                        }
                        Field::Mac => {
                            _mac_addr = map.next_value::<Option<String>>().ok();
                        }
                    }
                }

                Ok(NetworkIfaceWrapper(NetworkInterface::new_afinet(
                    name.unwrap().as_str(),
                    addr.unwrap(),
                    None,
                    None,
                    index.unwrap(),
                )))
            }
        }

        const FIELDS: &[&str] = &["name", "index", "addr", "mac"];
        deserializer.deserialize_struct("NetworkInterface", FIELDS, NetworkIfaceVisitor)
    }
}

fn main() {
    let mut info_file = NamedTempFile::new().unwrap();
    let mut stored_iface: Option<NetworkIfaceWrapper> = None;

    dbg!(info_file.as_ref().as_os_str());

    loop {
        if stored_iface.is_none() {
            info_file.rewind().unwrap();
            let helper_fd = info_file.reopen().unwrap();

            if let Ok(metadata) = helper_fd.metadata() {
                let file_size = metadata.len();

                if file_size > 0 {
                    stored_iface = serde_json::from_reader(helper_fd).ok();
                }
            }
        }

        let net = NetworkInterface::show();

        for iface in net.unwrap().iter().filter(|it_iface| {
            it_iface.name.starts_with("tun")
                && matches!(it_iface.addr, Some(network_interface::Addr::V4(_)))
        }) {
            let ser_iface = NetworkIfaceWrapper(iface.to_owned());

            if stored_iface.is_none() || stored_iface.as_ref().unwrap() != &ser_iface {
                println!(
                    "{} - {:?} - {:?}",
                    ser_iface.0.name, ser_iface.0.addr, ser_iface.0.mac_addr
                );
                serde_json::to_writer(&info_file, &ser_iface).unwrap();
                stored_iface = None;
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}
