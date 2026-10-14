use crate::{error::HardwareProbeError, types::HardwareProfile};
use sysinfo::{Disks, System};

pub(crate) struct CommonHardware {
    pub cpu_threads: usize,
    pub memory_bytes: u64,
    pub disk_available_bytes: u64,
}

pub(crate) fn collect_common() -> Result<CommonHardware, HardwareProbeError> {
    let system = System::new_all();
    let current_dir = std::env::current_dir().map_err(|error| {
        HardwareProbeError::DiskError(format!("cannot read current directory: {error}"))
    })?;
    let disks = Disks::new_with_refreshed_list();
    let disk = disks
        .list()
        .iter()
        .filter(|disk| current_dir.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().components().count())
        .ok_or_else(|| {
            HardwareProbeError::DiskError(format!(
                "no visible disk contains current directory {}",
                current_dir.display()
            ))
        })?;

    Ok(CommonHardware {
        cpu_threads: system.cpus().len(),
        memory_bytes: system.total_memory(),
        disk_available_bytes: disk.available_space(),
    })
}

impl CommonHardware {
    pub(crate) fn with_accelerators(
        self,
        accelerators: Vec<crate::types::AcceleratorProfile>,
    ) -> HardwareProfile {
        HardwareProfile {
            cpu_threads: self.cpu_threads,
            memory_bytes: self.memory_bytes,
            disk_available_bytes: self.disk_available_bytes,
            accelerators,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_hardware_reports_nonzero_cpu_count() {
        let common = collect_common().expect("common hardware collection should succeed");
        assert!(common.cpu_threads > 0);
    }
}
