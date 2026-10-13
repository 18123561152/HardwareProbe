use crate::{error::HardwareProbeError, types::HardwareProfile};

/// Fallback probe compiled for targets without a platform implementation.
pub struct SystemHardwareProbe;

impl SystemHardwareProbe {
    pub fn new() -> Self {
        Self
    }

    pub async fn inspect(&self) -> Result<HardwareProfile, HardwareProbeError> {
        Err(HardwareProbeError::UnsupportedPlatform(
            std::env::consts::OS.to_owned(),
        ))
    }
}
