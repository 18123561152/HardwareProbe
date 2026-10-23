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
        (AcceleratorKind::Other, "\"Other\""),
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
    let system = sysinfo::System::new_all();
    let expected_cpu = system
        .cpus()
        .first()
        .map(|cpu| cpu.brand().trim())
        .filter(|brand| !brand.is_empty())
        .unwrap_or("Unknown CPU");
    let result = SystemHardwareProbe::new().inspect().await;

    if detect_platform() == Platform::Other {
        assert!(matches!(
            result,
            Err(HardwareProbeError::UnsupportedPlatform(_))
        ));
        return;
    }

    let profile = result.expect("supported platforms must return common hardware facts");
    assert_eq!(profile.cpu, expected_cpu);
    assert!(!profile.cpu.trim().is_empty());
    assert!(!profile
        .accelerators
        .iter()
        .any(|accelerator| accelerator.name.trim().is_empty()));
    let serialized = serde_json::to_value(profile).unwrap();
    assert!(serialized["cpu"].is_string());
    assert!(serialized.get("cpu_threads").is_none());
    for accelerator in serialized["accelerators"]
        .as_array()
        .expect("accelerators must serialize as an array")
    {
        let kind = accelerator["kind"]
            .as_str()
            .expect("accelerator kind must serialize as a string");
        assert!(
            matches!(kind, "Nvidia" | "Amd" | "Intel" | "Apple" | "Other"),
            "unexpected accelerator kind: {kind}"
        );
    }
}
