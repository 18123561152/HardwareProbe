use std::process::Command;

use serde_json::Value;

use crate::{
    error::HardwareProbeError,
    types::{AcceleratorKind, AcceleratorProfile, HardwareProfile},
};

use super::common::collect_common;

pub struct SystemHardwareProbe;

impl SystemHardwareProbe {
    pub fn new() -> Self {
        Self
    }

    pub async fn inspect(&self) -> Result<HardwareProfile, HardwareProbeError> {
        tokio::task::spawn_blocking(|| {
            let common = collect_common()?;
            // A missing or failing system_profiler only disables accelerator reporting.
            let accelerators = detect_accelerators().unwrap_or_default();
            Ok(common.with_accelerators(accelerators))
        })
        .await
        .map_err(|error| HardwareProbeError::DetectionFailed(error.to_string()))?
    }
}

fn detect_accelerators() -> Result<Vec<AcceleratorProfile>, HardwareProbeError> {
    let output = Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output()
        .map_err(|error| HardwareProbeError::GpuDriverUnavailable(error.to_string()))?;
    if !output.status.success() {
        return Err(HardwareProbeError::GpuDriverUnavailable(format!(
            "system_profiler exited with {}",
            output.status
        )));
    }
    let value: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| HardwareProbeError::GpuDriverUnavailable(error.to_string()))?;
    Ok(parse_displays(&value))
}

fn parse_displays(value: &Value) -> Vec<AcceleratorProfile> {
    let Some(displays) = value.get("SPDisplaysDataType").and_then(Value::as_array) else {
        return Vec::new();
    };

    displays
        .iter()
        .filter_map(|display| {
            let vendor = first_string(
                display,
                &["spdisplays_vendor", "sppci_vendor", "spdisplays_gpu_vendor"],
            )?;
            let kind = classify_vendor(&vendor)?;
            let name = first_string(display, &["sppci_model", "_name", "spdisplays_name"])
                .unwrap_or_else(|| format!("{vendor} GPU"));

            // system_profiler may report VRAM as a human-readable string; don't guess units.
            // Apple Silicon shares system memory and therefore has no dedicated VRAM value.
            let memory_bytes = if kind == AcceleratorKind::Apple {
                None
            } else {
                display.get("spdisplays_vram").and_then(parse_byte_count)
            };

            Some(AcceleratorProfile {
                kind,
                name,
                memory_bytes,
            })
        })
        .collect()
}

fn first_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn classify_vendor(vendor: &str) -> Option<AcceleratorKind> {
    let vendor = vendor.to_ascii_lowercase();
    if vendor.contains("apple") || vendor.contains("0x106b") {
        Some(AcceleratorKind::Apple)
    } else if vendor.contains("amd") || vendor.contains("ati") || vendor.contains("0x1002") {
        Some(AcceleratorKind::Amd)
    } else if vendor.contains("intel") || vendor.contains("0x8086") {
        Some(AcceleratorKind::Intel)
    } else if vendor.contains("nvidia") || vendor.contains("0x10de") {
        Some(AcceleratorKind::Nvidia)
    } else {
        None
    }
}

fn parse_byte_count(value: &Value) -> Option<u64> {
    if let Some(bytes) = value.as_u64() {
        return Some(bytes);
    }

    let text = value.as_str()?.trim();
    let mut parts = text.split_whitespace();
    let number = parts.next()?.replace(',', "").parse::<f64>().ok()?;
    let unit = parts.next()?.to_ascii_lowercase();
    let multiplier = match unit.as_str() {
        "b" | "byte" | "bytes" => 1_u64,
        "kb" | "kib" => 1_u64 << 10,
        "mb" | "mib" => 1_u64 << 20,
        "gb" | "gib" => 1_u64 << 30,
        "tb" | "tib" => 1_u64 << 40,
        _ => return None,
    };
    let bytes = number * multiplier as f64;
    (bytes.is_finite() && bytes >= 0.0 && bytes <= u64::MAX as f64).then_some(bytes.round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn apple_gpu_does_not_claim_unified_memory_as_vram() {
        let displays = json!({
            "SPDisplaysDataType": [{
                "spdisplays_vendor": "Apple (0x106b)",
                "sppci_model": "Apple M-series GPU",
                "spdisplays_vram": "16 GB"
            }]
        });
        let parsed = parse_displays(&displays);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].kind, AcceleratorKind::Apple);
        assert_eq!(parsed[0].memory_bytes, None);
    }

    #[test]
    fn vendor_classifier_handles_common_vendor_ids() {
        assert_eq!(classify_vendor("AMD (0x1002)"), Some(AcceleratorKind::Amd));
        assert_eq!(
            classify_vendor("Intel (0x8086)"),
            Some(AcceleratorKind::Intel)
        );
    }

    #[test]
    fn parses_system_profiler_vram_units() {
        assert_eq!(parse_byte_count(&json!("8 GB")), Some(8_u64 << 30));
        assert_eq!(parse_byte_count(&json!(4096_u64)), Some(4096));
        assert_eq!(parse_byte_count(&json!("shared")), None);
    }
}
