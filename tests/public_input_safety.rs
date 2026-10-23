use hardware_probe::{AcceleratorKind, AcceleratorProfile, HardwareProfile};
use serde_json::{json, Value};

fn deserialize_profile(value: Value) -> Result<HardwareProfile, serde_json::Error> {
    serde_json::from_value(value)
}

#[test]
fn profile_accepts_numeric_boundaries_and_unknown_memory() {
    let value = json!({
        "cpu": "Example CPU Model",
        "memory_bytes": u64::MAX,
        "disk_available_bytes": 0,
        "accelerators": [{
            "kind": "Other",
            "name": "Unidentified accelerator",
            "memory_bytes": 0
        }]
    });
    let profile = deserialize_profile(value).expect("valid boundary values should deserialize");

    assert_eq!(profile.cpu, "Example CPU Model");
    assert_eq!(profile.memory_bytes, u64::MAX);
    assert_eq!(profile.disk_available_bytes, 0);
    assert_eq!(
        profile.accelerators,
        vec![AcceleratorProfile {
            kind: AcceleratorKind::Other,
            name: "Unidentified accelerator".to_owned(),
            memory_bytes: Some(0),
        }]
    );
}

#[test]
fn other_accelerator_is_a_valid_public_input() {
    let value = json!({
        "cpu": "Example CPU Model",
        "memory_bytes": 1024,
        "disk_available_bytes": 2048,
        "accelerators": [{
            "kind": "Other",
            "name": "Vendor-neutral GPU",
            "memory_bytes": null
        }]
    });

    let profile = deserialize_profile(value).expect("Other should be accepted");
    assert_eq!(profile.accelerators[0].kind, AcceleratorKind::Other);
}

#[test]
fn profile_rejects_missing_or_wrongly_typed_fields() {
    let valid = json!({
        "cpu": "Example CPU Model",
        "memory_bytes": 1024,
        "disk_available_bytes": 2048,
        "accelerators": []
    });

    for invalid in [
        json!({}),
        json!({
            "memory_bytes": 1024,
            "disk_available_bytes": 2048,
            "accelerators": []
        }),
        json!({
            "cpu": 4,
            "memory_bytes": 1024,
            "disk_available_bytes": 2048,
            "accelerators": []
        }),
        json!({
            "cpu": "Example CPU Model",
            "memory_bytes": -1,
            "disk_available_bytes": 2048,
            "accelerators": []
        }),
        json!({
            "cpu": "Example CPU Model",
            "memory_bytes": 1024,
            "disk_available_bytes": "2048",
            "accelerators": []
        }),
        json!({
            "cpu": "Example CPU Model",
            "memory_bytes": 1024,
            "disk_available_bytes": 2048,
            "accelerators": null
        }),
    ] {
        assert!(
            deserialize_profile(invalid).is_err(),
            "invalid profile unexpectedly accepted"
        );
    }

    assert!(deserialize_profile(valid).is_ok());
}

#[test]
fn unknown_cpu_fallback_is_a_valid_public_value() {
    let value = json!({
        "cpu": "Unknown CPU",
        "memory_bytes": 0,
        "disk_available_bytes": 0,
        "accelerators": []
    });
    let profile = deserialize_profile(value).expect("fallback CPU model should be valid");
    assert_eq!(profile.cpu, "Unknown CPU");
}

#[test]
fn accelerator_rejects_unknown_kinds_and_invalid_memory_types() {
    for accelerator in [
        json!({"kind": "Cuda", "name": "GPU", "memory_bytes": 1024}),
        json!({"kind": "Nvidia", "name": "GPU", "memory_bytes": -1}),
        json!({"kind": "Nvidia", "name": "GPU", "memory_bytes": "1024"}),
        json!({"kind": "Nvidia", "name": 7, "memory_bytes": null}),
    ] {
        let value = json!({
            "cpu": "Example CPU Model",
            "memory_bytes": 1,
            "disk_available_bytes": 1,
            "accelerators": [accelerator]
        });
        assert!(
            deserialize_profile(value).is_err(),
            "invalid accelerator unexpectedly accepted"
        );
    }
}

#[test]
fn out_of_range_json_integers_are_rejected() {
    let input = r#"{
        "cpu": "Example CPU Model",
        "memory_bytes": 18446744073709551616,
        "disk_available_bytes": 1,
        "accelerators": []
    }"#;
    assert!(serde_json::from_str::<HardwareProfile>(input).is_err());
}
