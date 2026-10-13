use serde::{Deserialize, Serialize};

/// Hardware facts visible to the current process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareProfile {
    /// Number of logical CPU threads.
    pub cpu_threads: usize,
    /// Total physical memory, in bytes.
    pub memory_bytes: u64,
    /// Sum of available space across all visible disks, in bytes.
    pub disk_available_bytes: u64,
    /// Detected accelerators. This can be empty when GPU probing is unavailable.
    pub accelerators: Vec<AcceleratorProfile>,
}

/// Information about one graphics or compute accelerator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceleratorProfile {
    pub kind: AcceleratorKind,
    pub name: String,
    /// Dedicated VRAM in bytes, or `None` when it cannot be reported separately.
    pub memory_bytes: Option<u64>,
}

/// Supported accelerator vendors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcceleratorKind {
    Nvidia,
    Amd,
    Intel,
    Apple,
}

/// Operating system represented by the current compilation target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
    Other,
}
