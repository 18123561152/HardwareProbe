use hardware_probe::{detect_platform, HardwareProbeError, Platform, SystemHardwareProbe};
use std::path::Path;
use sysinfo::Disks;

#[tokio::test]
async fn inspect_reports_only_the_current_directory_filesystem() {
    let result = SystemHardwareProbe::new().inspect().await;

    if detect_platform() == Platform::Other {
        assert!(matches!(
            result,
            Err(HardwareProbeError::UnsupportedPlatform(_))
        ));
        return;
    }

    let current_dir = std::env::current_dir().expect("test working directory must be readable");
    let disks = Disks::new_with_refreshed_list();
    let expected = disks
        .list()
        .iter()
        .filter(|disk| current_dir.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().components().count())
        .map(|disk| disk.available_space())
        .expect("current directory must map to a visible disk");
    let all_disk_total = disks
        .list()
        .iter()
        .map(|disk| disk.available_space())
        .fold(0_u64, u64::saturating_add);
    let profile = result.expect("supported platform must report disk availability");

    assert_eq!(profile.disk_available_bytes, expected);
    if all_disk_total != expected {
        assert_ne!(profile.disk_available_bytes, all_disk_total);
    }
}

#[test]
fn mount_matching_respects_path_component_boundaries() {
    #[cfg(unix)]
    {
        assert!(Path::new("/data/project").starts_with(Path::new("/data")));
        assert!(!Path::new("/database/project").starts_with(Path::new("/data")));
    }

    #[cfg(windows)]
    {
        assert!(Path::new(r"C:\data\project").starts_with(Path::new(r"C:\data")));
        assert!(!Path::new(r"C:\database\project").starts_with(Path::new(r"C:\data")));
    }
}
