use hardware_probe::{AcceleratorKind, AcceleratorProfile, HardwareProfile};
use serde_json::{json, Value};

fn deserialize_profile(value: Value) -> Result<HardwareProfile, serde_json::Error> {
    serde_json::from_value(value)
}

#[test]
fn profile_accepts_numeric_boundaries_and_unknown_memory() {
    let value = json!({
        "cpu_threads": usize::MAX,
        "memory_bytes": u64::MAX,
        "disk_available_bytes": 0,
        "accelerators": [{
            "kind": "Apple",
            "name": "",
            "memory_bytes": null
        }]
    });
    let profile = deserialize_profile(value).expect("valid boundary values should deserialize");

    assert_eq!(profile.cpu_threads, usize::MAX);
    assert_eq!(profile.memory_bytes, u64::MAX);
    assert_eq!(profile.disk_available_bytes, 0);
    assert_eq!(
        profile.accelerators,
        vec![AcceleratorProfile {
            kind: AcceleratorKind::Apple,
            name: String::new(),
            memory_bytes: None,
        }]
    );
}

#[test]
fn profile_rejects_missing_or_wrongly_typed_fields() {
    let valid = json!({
        "cpu_threads": 4,
        "memory_bytes": 1024,
        "disk_available_bytes": 2048,
        "accelerators": []
    });

    for invalid in [
        json!({}),
        json!({
            "cpu_threads": -1,
            "memory_bytes": 1024,
            "disk_available_bytes": 2048,
            "accelerators": []
        }),
        json!({
            "cpu_threads": 4,
            "memory_bytes": -1,
            "disk_available_bytes": 2048,
            "accelerators": []
        }),
        json!({
            "cpu_threads": 4,
            "memory_bytes": 1024,
            "disk_available_bytes": "2048",
            "accelerators": []
        }),
        json!({
            "cpu_threads": 4,
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
fn accelerator_rejects_unknown_kinds_and_invalid_memory_types() {
    for accelerator in [
        json!({"kind": "Cuda", "name": "GPU", "memory_bytes": 1024}),
        json!({"kind": "Nvidia", "name": "GPU", "memory_bytes": -1}),
        json!({"kind": "Nvidia", "name": "GPU", "memory_bytes": "1024"}),
        json!({"kind": "Nvidia", "name": 7, "memory_bytes": null}),
    ] {
        let value = json!({
            "cpu_threads": 1,
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
        "cpu_threads": 1,
        "memory_bytes": 18446744073709551616,
        "disk_available_bytes": 1,
        "accelerators": []
    }"#;
    assert!(serde_json::from_str::<HardwareProfile>(input).is_err());
}
