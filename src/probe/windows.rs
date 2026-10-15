use std::process::Command;

use serde_json::Value;

use crate::{
    error::HardwareProbeError,
    types::{AcceleratorKind, AcceleratorProfile, HardwareProfile},
};
use nvml_wrapper::Nvml;

use super::common::collect_common;

pub struct SystemHardwareProbe;

impl SystemHardwareProbe {
    pub fn new() -> Self {
        Self
    }

    pub async fn inspect(&self) -> Result<HardwareProfile, HardwareProbeError> {
        tokio::task::spawn_blocking(|| {
            let common = collect_common()?;
            let mut accelerators = detect_nvidia().unwrap_or_default();
            accelerators.extend(detect_cim_accelerators().unwrap_or_default());
            Ok(common.with_accelerators(accelerators))
        })
        .await
        .map_err(|error| HardwareProbeError::DetectionFailed(error.to_string()))?
    }
}

fn detect_nvidia() -> Result<Vec<AcceleratorProfile>, HardwareProbeError> {
    let nvml = Nvml::init()
        .map_err(|error| HardwareProbeError::GpuDriverUnavailable(error.to_string()))?;
    let count = nvml
        .device_count()
        .map_err(|error| HardwareProbeError::GpuDriverUnavailable(error.to_string()))?;

    let mut devices = Vec::new();
    for index in 0..count {
        let Ok(device) = nvml.device_by_index(index) else {
            continue;
        };
        let name = device
            .name()
            .unwrap_or_else(|_| format!("NVIDIA GPU {index}"));
        let memory_bytes = device.memory_info().ok().map(|memory| memory.total);
        devices.push(AcceleratorProfile {
            kind: AcceleratorKind::Nvidia,
            name,
            memory_bytes,
        });
    }
    Ok(devices)
}

fn detect_cim_accelerators() -> Result<Vec<AcceleratorProfile>, HardwareProbeError> {
    let script = "Get-CimInstance Win32_VideoController | Select-Object Name, AdapterCompatibility, AdapterRAM | ConvertTo-Json -Compress";
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|error| HardwareProbeError::GpuDriverUnavailable(error.to_string()))?;
    if !output.status.success() {
        return Err(HardwareProbeError::GpuDriverUnavailable(format!(
            "PowerShell exited with {}",
            output.status
        )));
    }
    let value: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| HardwareProbeError::GpuDriverUnavailable(error.to_string()))?;
    Ok(parse_cim_devices(&value))
}

fn parse_cim_devices(value: &Value) -> Vec<AcceleratorProfile> {
    let records: Vec<&Value> = match value {
        Value::Array(values) => values.iter().collect(),
        Value::Object(_) => vec![value],
        _ => Vec::new(),
    };

    records
        .into_iter()
        .filter_map(|record| {
            let name = record.get("Name")?.as_str()?.trim();
            if name.is_empty() || name.to_ascii_lowercase().contains("nvidia") {
                return None;
            }

            let compatibility = record
                .get("AdapterCompatibility")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let kind = classify_vendor(name).or_else(|| classify_vendor(compatibility))?;
            Some(AcceleratorProfile {
                kind,
                name: name.to_owned(),
                memory_bytes: record.get("AdapterRAM").and_then(parse_adapter_ram),
            })
        })
        .collect()
}

fn classify_vendor(vendor: &str) -> Option<AcceleratorKind> {
    let vendor = vendor.to_ascii_lowercase();
    if vendor.contains("amd") || vendor.contains("advanced micro devices") || vendor.contains("ati")
    {
        Some(AcceleratorKind::Amd)
    } else if vendor.contains("intel") {
        Some(AcceleratorKind::Intel)
    } else if vendor.contains("apple") {
        Some(AcceleratorKind::Apple)
    } else {
        None
    }
}

fn parse_adapter_ram(value: &Value) -> Option<u64> {
    let bytes = value.as_u64()?;
    // Win32_VideoController.AdapterRAM is a 32-bit field; its maximum commonly signals overflow.
    if bytes == 0 || bytes >= u32::MAX as u64 {
        None
    } else {
        Some(bytes)
    }
}
