use assert_cmd::Command;

#[test]
fn cli_json_outputs_parseable_json_only() {
    let output = Command::cargo_bin("pi-birthday-bench")
        .expect("binary exists")
        .args([
            "--target",
            "20240628",
            "--max-digits",
            "100",
            "--backend",
            "cpu-single",
            "--json",
        ])
        .output()
        .expect("command runs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(
        !stdout.contains("target:"),
        "stdout must not contain human-readable result text"
    );

    let value: serde_json::Value = serde_json::from_str(&stdout).expect("stdout is JSON");

    assert_eq!(value["target"], "20240628");
    assert_eq!(value["backend"], "cpu-single");
    assert_eq!(value["algorithm"], "chudnovsky_binary_splitting");
    assert_eq!(value["digits_computed"], 100);
    assert_eq!(value["chunks_processed"], 1);
    assert!(value.get("elapsed_seconds").is_some());
    assert!(value.get("digits_per_second").is_some());
    assert!(value["threads"].is_null());
    assert!(value["cpu_model"].is_null());
    assert!(value["gpu_name"].is_null());
    assert_eq!(value["gpu_role"], "none");
    assert!(value["memory_peak_mb"].is_null());
    assert_eq!(value["verification_status"], "skipped");
}
