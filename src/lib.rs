//! Cross-platform collection of basic host hardware facts.

pub mod error;
pub mod probe;
pub mod types;

pub use error::HardwareProbeError;
pub use probe::SystemHardwareProbe;
pub use types::{AcceleratorKind, AcceleratorProfile, HardwareProfile, Platform};

/// Return the platform selected by the current compilation target.
pub const fn detect_platform() -> Platform {
    #[cfg(target_os = "linux")]
    {
        Platform::Linux
    }
    #[cfg(target_os = "macos")]
    {
        Platform::MacOS
    }
    #[cfg(target_os = "windows")]
    {
        Platform::Windows
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Platform::Other
    }
}

/// Return Rust's compile-time operating-system identifier.
pub const fn current_os() -> &'static str {
    std::env::consts::OS
}

/// Return Rust's compile-time architecture identifier.
pub const fn current_arch() -> &'static str {
    std::env::consts::ARCH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_helpers_match_compile_target() {
        assert!(!current_os().is_empty());
        assert!(!current_arch().is_empty());

        #[cfg(target_os = "windows")]
        assert_eq!(detect_platform(), Platform::Windows);
        #[cfg(target_os = "linux")]
        assert_eq!(detect_platform(), Platform::Linux);
        #[cfg(target_os = "macos")]
        assert_eq!(detect_platform(), Platform::MacOS);
    }

    #[test]
    fn hardware_profile_uses_contract_field_names() {
        let profile = HardwareProfile {
            cpu_threads: 8,
            memory_bytes: 16_000,
            disk_available_bytes: 8_000,
            accelerators: vec![AcceleratorProfile {
                kind: AcceleratorKind::Apple,
                name: "Apple GPU".to_owned(),
                memory_bytes: None,
            }],
        };
        let value = serde_json::to_value(profile).unwrap();
        assert_eq!(value["cpu_threads"], 8);
        assert_eq!(value["memory_bytes"], 16_000);
        assert_eq!(value["disk_available_bytes"], 8_000);
        assert_eq!(value["accelerators"][0]["kind"], "Apple");
        assert_eq!(
            value["accelerators"][0]["memory_bytes"],
            serde_json::Value::Null
        );
    }
}
