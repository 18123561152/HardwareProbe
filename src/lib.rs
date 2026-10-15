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
