use hardware_probe::{
    current_arch, current_os, detect_platform, AcceleratorKind, HardwareProbeError, Platform,
    SystemHardwareProbe,
};

#[test]
fn platform_helpers_match_the_compilation_target() {
    assert_eq!(current_os(), std::env::consts::OS);
    assert_eq!(current_arch(), std::env::consts::ARCH);

    let expected = match std::env::consts::OS {
        "linux" => Platform::Linux,
        "macos" => Platform::MacOS,
        "windows" => Platform::Windows,
        _ => Platform::Other,
    };
    assert_eq!(detect_platform(), expected);
}

#[test]
fn public_enums_round_trip_through_serde() {
    for (kind, encoded) in [
        (AcceleratorKind::Nvidia, "\"Nvidia\""),
        (AcceleratorKind::Amd, "\"Amd\""),
        (AcceleratorKind::Intel, "\"Intel\""),
        (AcceleratorKind::Apple, "\"Apple\""),
    ] {
        assert_eq!(serde_json::to_string(&kind).unwrap(), encoded);
        assert_eq!(
            serde_json::from_str::<AcceleratorKind>(encoded).unwrap(),
            kind
        );
    }

    for (platform, encoded) in [
        (Platform::Linux, "\"Linux\""),
        (Platform::MacOS, "\"MacOS\""),
        (Platform::Windows, "\"Windows\""),
        (Platform::Other, "\"Other\""),
    ] {
        assert_eq!(serde_json::to_string(&platform).unwrap(), encoded);
        assert_eq!(serde_json::from_str::<Platform>(encoded).unwrap(), platform);
    }
}

#[test]
fn public_errors_format_untrusted_messages_without_panicking() {
    let hostile_message = "gpu\0driver\n\u{1b}[31m".repeat(128);
    let errors = [
        HardwareProbeError::DetectionFailed(hostile_message.clone()),
        HardwareProbeError::GpuDriverUnavailable(hostile_message.clone()),
        HardwareProbeError::DiskError(hostile_message.clone()),
        HardwareProbeError::UnsupportedPlatform(hostile_message.clone()),
    ];

    for error in errors {
        let rendered = error.to_string();
        assert!(rendered.contains(&hostile_message));
    }
}

#[tokio::test]
async fn inspect_obeys_the_compiled_platform_contract() {
    let result = SystemHardwareProbe::new().inspect().await;

    if detect_platform() == Platform::Other {
        assert!(matches!(
            result,
            Err(HardwareProbeError::UnsupportedPlatform(_))
        ));
        return;
    }

    let profile = result.expect("supported platforms must return common hardware facts");
    assert!(profile.cpu_threads > 0);
    assert!(!profile
        .accelerators
        .iter()
        .any(|accelerator| accelerator.name.trim().is_empty()));
}
