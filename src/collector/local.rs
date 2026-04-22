use std::time::SystemTime;

use anyhow::Result;
use sysinfo::{ComponentExt, CpuExt, System, SystemExt};

use crate::{
    collector::{disks::collect_local_physical_disks, HostCollector},
    config::AppConfig,
    model::{HostDescriptor, HostInfo, HostStatus, HostType, MetricsSnapshot},
};

pub struct LocalCollector {
    descriptor: HostDescriptor,
    system: System,
}

impl LocalCollector {
    pub fn new(_config: &AppConfig) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let hostname = system.host_name().unwrap_or_else(|| "localhost".to_string());
        Self {
            descriptor: HostDescriptor {
                id: "local".to_string(),
                alias: "local".to_string(),
                display_name: hostname,
                host_type: HostType::Local,
            },
            system,
        }
    }

    fn cpu_temperature_celsius(&self) -> Option<f64> {
        self.system
            .components()
            .iter()
            .filter_map(|component| {
                let temp = component.temperature();
                temp.is_finite().then_some((component.label(), temp as f64))
            })
            .max_by_key(|(label, _)| cpu_component_score(label))
            .map(|(_, temp)| temp)
    }
}

impl HostCollector for LocalCollector {
    fn descriptor(&self) -> HostDescriptor {
        self.descriptor.clone()
    }

    fn collect(&mut self) -> Result<HostInfo> {
        self.system.refresh_cpu();
        self.system.refresh_memory();
        self.system.refresh_components();

        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let memory_usage_percent = if total_memory == 0 {
            0.0
        } else {
            used_memory as f64 * 100.0 / total_memory as f64
        };

        let disks = collect_local_physical_disks()?;

        Ok(HostInfo {
            id: self.descriptor.id.clone(),
            alias: self.descriptor.alias.clone(),
            display_name: self.descriptor.display_name.clone(),
            host_type: self.descriptor.host_type,
            status: HostStatus::Online,
            metrics: MetricsSnapshot {
                cpu_usage_percent: self.system.global_cpu_info().cpu_usage() as f64,
                cpu_temperature_celsius: self.cpu_temperature_celsius(),
                memory_used_bytes: used_memory,
                memory_total_bytes: total_memory,
                memory_usage_percent,
                disks,
            },
            last_updated: Some(SystemTime::now()),
            last_error: None,
        })
    }
}

fn cpu_component_score(label: &str) -> usize {
    let label = label.to_ascii_lowercase();

    if label.contains("package id") || label.contains("tctl") || label.contains("tdie") {
        return 4;
    }
    if label.contains("cpu") || label.contains("core") || label.contains("k10temp") {
        return 3;
    }
    if label.contains("ccd") {
        return 2;
    }

    1
}
