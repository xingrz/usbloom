use anyhow::{Context, Result, bail};
use cyme::profiler::{Device, SystemProfile};
use cyme::usb::{Configuration, Interface, Speed};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::Path,
    time::SystemTime,
};

const SNAPSHOT_LIMIT: u64 = 16 * 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub schema_version: u32,
    pub captured_at: u64,
    pub profile: SystemProfile,
    // cyme deliberately omits profiler errors from its JSON representation.
    pub read_errors: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct DeviceRow {
    pub key: String,
    pub parent: Option<String>,
    pub bus: String,
    pub bus_key: String,
    pub depth: usize,
    pub has_children: bool,
    pub device: Device,
}

impl Snapshot {
    pub fn capture() -> Result<Self> {
        let profile = cyme::profiler::get_spusb_with_options(&cyme::profiler::ProfilerOptions {
            tree: true,
            depth: cyme::profiler::ProfileDepth::Full,
            ..Default::default()
        })
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        Ok(Self::from_profile(profile))
    }

    pub fn from_profile(profile: SystemProfile) -> Self {
        let read_errors = profile
            .flattened_devices()
            .into_iter()
            .filter_map(|device| {
                device
                    .profiler_error
                    .as_ref()
                    .map(|error| (device_key(device), error.clone()))
            })
            .collect();
        Self {
            schema_version: 1,
            captured_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            profile,
            read_errors,
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .context("Could not open snapshot")?
            .take(SNAPSHOT_LIMIT + 1)
            .read_to_end(&mut bytes)?;
        Self::decode(&bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() as u64 > SNAPSHOT_LIMIT {
            bail!("Snapshot exceeds 16 MB");
        }
        let snapshot: Self = serde_json::from_slice(bytes).context("Not a USBloom snapshot")?;
        if snapshot.schema_version != 1 {
            bail!("Unsupported snapshot version {}", snapshot.schema_version);
        }
        fn count(devices: &[Device], depth: usize, total: &mut usize) -> Result<()> {
            if depth > 32 {
                bail!("Snapshot device tree is too deep");
            }
            for device in devices {
                *total += 1;
                if *total > 10_000 {
                    bail!("Snapshot contains too many devices");
                }
                count(
                    device.devices.as_deref().unwrap_or_default(),
                    depth + 1,
                    total,
                )?;
            }
            Ok(())
        }
        let mut total = 0;
        for bus in &snapshot.profile.buses {
            count(bus.devices.as_deref().unwrap_or_default(), 0, &mut total)?;
        }
        let mut keys = std::collections::HashSet::new();
        for row in snapshot.rows() {
            if !keys.insert(row.key) {
                bail!("Snapshot contains duplicate device identities");
            }
        }
        Ok(snapshot)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(self)?;
        if bytes.len() as u64 > SNAPSHOT_LIMIT {
            bail!("Snapshot exceeds 16 MB");
        }
        let folder = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut file = tempfile::NamedTempFile::new_in(folder)?;
        file.write_all(&bytes)?;
        file.as_file().sync_all()?;
        file.persist(path).context("Could not save snapshot")?;
        Ok(())
    }

    pub fn rows(&self) -> Vec<DeviceRow> {
        fn walk(
            devices: &[Device],
            bus: &str,
            bus_key: &str,
            depth: usize,
            parent: Option<&str>,
            rows: &mut Vec<DeviceRow>,
        ) {
            for device in devices {
                let key = device_key(device);
                let children = device.devices.as_deref().unwrap_or_default();
                let mut copy = device.clone();
                copy.devices = None;
                rows.push(DeviceRow {
                    key: key.clone(),
                    parent: parent.map(str::to_owned),
                    bus: bus.into(),
                    bus_key: bus_key.into(),
                    depth,
                    has_children: !children.is_empty(),
                    device: copy,
                });
                walk(children, bus, bus_key, depth + 1, Some(&key), rows);
            }
        }
        let mut rows = Vec::new();
        for (index, bus) in self.profile.buses.iter().enumerate() {
            let number = bus
                .usb_bus_number
                .map_or_else(|| index.to_string(), |n| n.to_string());
            let label = if bus.name.trim().is_empty() {
                format!("USB bus {number}")
            } else {
                bus.name.clone()
            };
            walk(
                bus.devices.as_deref().unwrap_or_default(),
                &label,
                &format!("bus-{number}"),
                0,
                None,
                &mut rows,
            );
        }
        rows
    }
}

pub fn device_key(device: &Device) -> String {
    // Device addresses change on re-enumeration; physical ports generally do not.
    format!(
        "{}:{:04x}:{:04x}:{}",
        device.port_path(),
        device.vendor_id.unwrap_or_default(),
        device.product_id.unwrap_or_default(),
        device.serial_num.as_deref().unwrap_or_default()
    )
}

pub fn visible_rows<'a>(
    rows: &'a [DeviceRow],
    query: &str,
    collapsed: &std::collections::HashSet<String>,
) -> Vec<&'a DeviceRow> {
    let query = query.trim().to_lowercase();
    let by_key: BTreeMap<&str, &DeviceRow> =
        rows.iter().map(|row| (row.key.as_str(), row)).collect();
    let mut included = std::collections::HashSet::new();
    if !query.is_empty() {
        for row in rows {
            let d = &row.device;
            let searchable = format!(
                "{} {} {} {} {} {}",
                d.name,
                d.manufacturer.as_deref().unwrap_or_default(),
                ids(d),
                d.serial_num.as_deref().unwrap_or_default(),
                d.port_path(),
                device_class(d)
            );
            if searchable.to_lowercase().contains(&query) {
                let mut current = Some(row);
                while let Some(node) = current {
                    included.insert(node.key.as_str());
                    current = node
                        .parent
                        .as_deref()
                        .and_then(|key| by_key.get(key).copied());
                }
            }
        }
    }
    rows.iter()
        .filter(|row| {
            if !query.is_empty() {
                return included.contains(row.key.as_str());
            }
            let mut parent = row.parent.as_deref();
            while let Some(key) = parent {
                if collapsed.contains(key) {
                    return false;
                }
                parent = by_key.get(key).and_then(|node| node.parent.as_deref());
            }
            true
        })
        .collect()
}

pub fn ids(device: &Device) -> String {
    format!("{} : {}", hex16(device.vendor_id), hex16(device.product_id))
}

pub fn hex16(value: Option<u16>) -> String {
    value.map_or_else(|| "Unknown".into(), |v| format!("{v:04X}"))
}

pub fn decoded(value: Option<u8>, name: Option<&str>) -> String {
    value.map_or_else(
        || "Unavailable".into(),
        |v| format!("{} · 0x{v:02X}", name.unwrap_or("Unassigned")),
    )
}

pub fn device_class(device: &Device) -> String {
    match device.base_class_code() {
        Some(0) => "Composite / per interface".into(),
        Some(0xff) => "Vendor specific".into(),
        _ => device.class_name().unwrap_or("USB device").into(),
    }
}

pub fn speed(device: &Device) -> String {
    match device
        .extra
        .as_ref()
        .and_then(|extra| extra.negotiated_speed.as_ref())
    {
        Some(Speed::LowSpeed) => "1.5 Mb/s",
        Some(Speed::FullSpeed) => "12 Mb/s",
        Some(Speed::HighSpeed | Speed::HighBandwidth) => "480 Mb/s",
        Some(Speed::SuperSpeed) => "5 Gb/s",
        Some(Speed::SuperSpeedPlus) => "10 Gb/s",
        Some(Speed::SuperSpeedPlusX2) => "20 Gb/s",
        Some(Speed::Usb40Gbps) => "40 Gb/s",
        Some(Speed::Usb80Gbps) => "80 Gb/s",
        _ => "Unavailable",
    }
    .into()
}

pub fn control_packet_size(device: &Device) -> String {
    let Some(extra) = &device.extra else {
        return "Unavailable".into();
    };
    if extra.max_packet_size == 0 {
        return "Unavailable".into();
    }
    let bytes = match device.bcd_usb {
        Some(version) if version.0 >= 3 => 1u32.checked_shl(extra.max_packet_size as u32),
        Some(_) => Some(extra.max_packet_size as u32),
        None => return format!("0x{:02X} (raw)", extra.max_packet_size),
    };
    bytes.map_or_else(|| "Invalid descriptor".into(), |n| format!("{n} bytes"))
}

pub fn interfaces(config: &Configuration) -> BTreeMap<u8, Vec<&Interface>> {
    let mut groups: BTreeMap<u8, Vec<&Interface>> = BTreeMap::new();
    for interface in &config.interfaces {
        groups.entry(interface.number).or_default().push(interface);
    }
    for alternatives in groups.values_mut() {
        alternatives.sort_by_key(|interface| interface.alt_setting);
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use cyme::profiler::{Bus, DeviceLocation};
    use std::collections::HashSet;

    fn device(name: &str, port: u8) -> Device {
        Device {
            name: name.into(),
            vendor_id: Some(0x1234),
            product_id: Some(0xabcd),
            location_id: DeviceLocation {
                bus: 1,
                number: 2,
                tree_positions: vec![port],
            },
            ..Default::default()
        }
    }
    fn sample() -> Snapshot {
        let mut hub = device("Example hub", 1);
        let mut child = device("Example MCU", 2);
        child.location_id.tree_positions = vec![1, 2];
        child.profiler_error = Some("Descriptor unavailable".into());
        hub.devices = Some(vec![child]);
        let mut bus = Bus::default();
        bus.name = "USB bus".into();
        bus.usb_bus_number = Some(1);
        bus.devices = Some(vec![hub]);
        Snapshot::from_profile(SystemProfile { buses: vec![bus] })
    }
    #[test]
    fn selection_identity_survives_address_change_but_not_device_replacement() {
        let a = device("Board", 2);
        let mut b = a.clone();
        b.location_id.number = 17;
        assert_eq!(device_key(&a), device_key(&b));
        b.product_id = Some(0xeeee);
        assert_ne!(device_key(&a), device_key(&b));
    }
    #[test]
    fn search_retains_ancestors_and_overrides_collapsed_branches() {
        let rows = sample().rows();
        let collapsed = HashSet::from([rows[0].key.clone()]);
        assert_eq!(visible_rows(&rows, "", &collapsed).len(), 1);
        assert_eq!(visible_rows(&rows, "mcu", &collapsed).len(), 2);
        assert!(visible_rows(&rows, "not here", &collapsed).is_empty());
    }
    #[test]
    fn snapshot_roundtrip_preserves_topology_and_partial_read_errors() {
        let original = sample();
        let loaded = Snapshot::decode(&serde_json::to_vec(&original).unwrap()).unwrap();
        assert_eq!(loaded.rows().len(), 2);
        assert_eq!(loaded.read_errors, original.read_errors);
        assert_eq!(loaded.rows()[1].parent, Some(loaded.rows()[0].key.clone()));
    }
    #[test]
    fn invalid_and_future_snapshots_are_rejected() {
        assert!(Snapshot::decode(b"{broken}").is_err());
        let mut future = sample();
        future.schema_version = 999;
        assert!(Snapshot::decode(&serde_json::to_vec(&future).unwrap()).is_err());
    }
    #[test]
    fn duplicate_identity_is_rejected_before_tree_navigation() {
        let mut snapshot = sample();
        let devices = snapshot.profile.buses[0].devices.as_mut().unwrap();
        devices.push(devices[0].clone());
        assert!(Snapshot::decode(&serde_json::to_vec(&snapshot).unwrap()).is_err());
    }
    #[test]
    fn saved_snapshot_preserves_speed_and_can_replace_a_previous_save() {
        let mut snapshot = sample();
        let device = &mut snapshot.profile.buses[0].devices.as_mut().unwrap()[0];
        device.extra = Some(
            serde_json::from_value(serde_json::json!({
                "max_packet_size": 9, "configurations": []
            }))
            .unwrap(),
        );
        device.extra.as_mut().unwrap().negotiated_speed = Some(Speed::SuperSpeed);
        device.bcd_usb = Some(cyme::usb::Version(3, 0, 0));
        assert_eq!(control_packet_size(device), "512 bytes");
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("snapshot.json");
        std::fs::write(&path, "previous content").unwrap();
        snapshot.save(&path).unwrap();
        let loaded = Snapshot::load(&path).unwrap();
        assert_eq!(speed(&loaded.rows()[0].device), "5 Gb/s");
        assert_eq!(control_packet_size(&loaded.rows()[0].device), "512 bytes");
    }
    #[test]
    fn unknown_is_not_reported_as_zero_or_full_speed() {
        let device = Device::default();
        assert_eq!(hex16(device.vendor_id), "Unknown");
        assert_eq!(speed(&device), "Unavailable");
        assert_eq!(decoded(None, None), "Unavailable");
    }
}
