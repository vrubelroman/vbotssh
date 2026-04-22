use std::process::Command;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::model::DiskInfo;

pub fn collect_local_physical_disks() -> Result<Vec<DiskInfo>> {
    let output = Command::new("lsblk")
        .args(["-J", "-b", "-o", "NAME,KNAME,PKNAME,PATH,TYPE,MOUNTPOINTS,SIZE,FSUSED"])
        .output()
        .context("failed to run lsblk")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "lsblk failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    parse_physical_disks_json(&String::from_utf8_lossy(&output.stdout))
}

pub fn parse_physical_disks_json(payload: &str) -> Result<Vec<DiskInfo>> {
    let snapshot: LsblkSnapshot = serde_json::from_str(payload).context("failed to parse lsblk JSON")?;

    Ok(snapshot
        .blockdevices
        .into_iter()
        .filter(|device| device.device_type == "disk")
        .filter_map(|device| disk_info_from_device(&device))
        .collect())
}

fn disk_info_from_device(device: &LsblkDevice) -> Option<DiskInfo> {
    let mut used_bytes = device.fsused.unwrap_or(0);
    let mut mountpoints = clean_mountpoints(&device.mountpoints);

    if let Some(children) = &device.children {
        for child in children {
            accumulate_child_usage(child, &mut used_bytes, &mut mountpoints);
        }
    }

    if mountpoints.is_empty() || device.size == 0 {
        return None;
    }

    Some(DiskInfo {
        name: device.name.clone(),
        mount_point: mountpoints.join(","),
        used_bytes,
        total_bytes: device.size,
        usage_percent: used_bytes as f64 * 100.0 / device.size as f64,
    })
}

fn accumulate_child_usage(device: &LsblkDevice, used_bytes: &mut u64, mountpoints: &mut Vec<String>) {
    if let Some(value) = device.fsused {
        *used_bytes = used_bytes.saturating_add(value);
    }

    for mountpoint in clean_mountpoints(&device.mountpoints) {
        if !mountpoints.iter().any(|existing| existing == &mountpoint) {
            mountpoints.push(mountpoint);
        }
    }

    if let Some(children) = &device.children {
        for child in children {
            accumulate_child_usage(child, used_bytes, mountpoints);
        }
    }
}

fn clean_mountpoints(mountpoints: &Option<Vec<Option<String>>>) -> Vec<String> {
    mountpoints
        .as_ref()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_ref())
                .map(|item| item.trim())
                .filter(|item| !item.is_empty() && *item != "[SWAP]")
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Debug, Deserialize)]
struct LsblkSnapshot {
    blockdevices: Vec<LsblkDevice>,
}

#[derive(Clone, Debug, Deserialize)]
struct LsblkDevice {
    name: String,
    #[serde(rename = "type")]
    device_type: String,
    size: u64,
    fsused: Option<u64>,
    mountpoints: Option<Vec<Option<String>>>,
    children: Option<Vec<LsblkDevice>>,
}
