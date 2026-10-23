use serde::{Deserialize, Serialize};

/// Hardware facts visible to the current process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareProfile {
    /// CPU model reported by the operating system.
    pub cpu: String,
    /// Total physical memory, in bytes.
    pub memory_bytes: u64,
    /// Available space on the filesystem containing the current working directory, in bytes.
    pub disk_available_bytes: u64,
    /// Detected accelerators. This can be empty when GPU probing is unavailable.
    pub accelerators: Vec<AcceleratorProfile>,
}

/// Information about one graphics or compute accelerator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceleratorProfile {
    pub kind: AcceleratorKind,
    pub name: String,
    /// GPU memory in bytes; on Apple Silicon this is the unified system memory total.
    /// `None` means the platform could not report a usable capacity.
    pub memory_bytes: Option<u64>,
}

/// Supported accelerator vendors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcceleratorKind {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Other,
}

/// Operating system represented by the current compilation target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
    Other,
}
