use crate::{error::HardwareProbeError, types::HardwareProfile};
use sysinfo::{Disks, System};

pub(crate) struct CommonHardware {
    pub cpu_threads: usize,
    pub memory_bytes: u64,
    pub disk_available_bytes: u64,
}

pub(crate) fn collect_common() -> Result<CommonHardware, HardwareProbeError> {
    let system = System::new_all();
    let disks = Disks::new_with_refreshed_list();
    let disk_available_bytes = disks.list().iter().try_fold(0_u64, |total, disk| {
        total.checked_add(disk.available_space()).ok_or_else(|| {
            HardwareProbeError::DiskError("sum of available disk space exceeds u64".to_owned())
        })
    })?;

    Ok(CommonHardware {
        cpu_threads: system.cpus().len(),
        memory_bytes: system.total_memory(),
        disk_available_bytes,
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
