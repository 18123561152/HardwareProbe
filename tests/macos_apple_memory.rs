#![cfg(target_os = "macos")]

use hardware_probe::{AcceleratorKind, SystemHardwareProbe};

#[tokio::test]
async fn apple_accelerators_report_total_system_memory() {
    let profile = SystemHardwareProbe::new()
        .inspect()
        .await
        .expect("macOS hardware probe should succeed");

    for accelerator in profile
        .accelerators
        .iter()
        .filter(|accelerator| accelerator.kind == AcceleratorKind::Apple)
    {
        assert_eq!(accelerator.memory_bytes, Some(profile.memory_bytes));
    }
}
