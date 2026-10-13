use thiserror::Error;

/// Errors returned by the hardware probe.
#[derive(Debug, Error)]
pub enum HardwareProbeError {
    #[error("hardware detection failed: {0}")]
    DetectionFailed(String),
    #[error("GPU driver is unavailable: {0}")]
    GpuDriverUnavailable(String),
    #[error("disk capacity could not be represented: {0}")]
    DiskError(String),
    #[error("unsupported platform: {0}")]
    UnsupportedPlatform(String),
}
