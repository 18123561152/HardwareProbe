#![cfg(unix)]

use hardware_probe::{HardwareProbeError, SystemHardwareProbe};
use std::{fs, time::SystemTime};

#[tokio::test]
async fn inspect_returns_disk_error_when_current_directory_is_unavailable() {
    let original_dir = std::env::current_dir().expect("original working directory");
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system clock must be after the Unix epoch")
        .as_nanos();
    let temporary_dir = std::env::temp_dir().join(format!(
        "hardware-probe-deleted-cwd-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir(&temporary_dir).expect("temporary directory should be created");
    std::env::set_current_dir(&temporary_dir).expect("temporary directory should be accessible");
    fs::remove_dir(&temporary_dir).expect("Unix permits removing the current directory");

    let result = SystemHardwareProbe::new().inspect().await;

    std::env::set_current_dir(original_dir).expect("restore original working directory");
    assert!(matches!(result, Err(HardwareProbeError::DiskError(_))));
}
