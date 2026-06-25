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
    assert!(
        output.stderr.is_empty(),
        "stderr must not contain progress when --json is set"
    );

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

#[test]
fn cli_json_accepts_benchmark_only() {
    let output = Command::cargo_bin("pi-birthday-bench")
        .expect("binary exists")
        .args([
            "--target",
            "20240628",
            "--max-digits",
            "100",
            "--chunk",
            "25",
            "--backend",
            "cpu-single",
            "--benchmark-only",
            "--json",
        ])
        .output()
        .expect("command runs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("stdout is JSON");

    assert_eq!(value["target"], "20240628");
    assert_eq!(value["digits_computed"], 100);
    assert_eq!(value["chunks_processed"], 4);
    assert!(value.get("digits_per_second").is_some());
}

#[test]
fn cpu_multi_json_matches_cpu_single_position_and_reports_threads() {
    let single = run_json([
        "--target",
        "20240628",
        "--max-digits",
        "100",
        "--backend",
        "cpu-single",
        "--json",
    ]);
    let multi = run_json([
        "--target",
        "20240628",
        "--max-digits",
        "100",
        "--backend",
        "cpu-multi",
        "--threads",
        "2",
        "--json",
    ]);

    assert_eq!(multi["backend"], "cpu-multi");
    assert_eq!(multi["threads"], 2);
    assert_eq!(multi["first_position"], single["first_position"]);
    assert_eq!(multi["found"], single["found"]);
}

#[test]
fn cpu_multi_threads_one_matches_cpu_single_position() {
    let single = run_json([
        "--target",
        "20240628",
        "--max-digits",
        "100",
        "--backend",
        "cpu-single",
        "--json",
    ]);
    let multi = run_json([
        "--target",
        "20240628",
        "--max-digits",
        "100",
        "--backend",
        "cpu-multi",
        "--threads",
        "1",
        "--json",
    ]);

    assert_eq!(multi["backend"], "cpu-multi");
    assert_eq!(multi["threads"], 1);
    assert_eq!(multi["first_position"], single["first_position"]);
    assert_eq!(multi["found"], single["found"]);
}

#[test]
fn verify_sets_json_status_to_passed() {
    let value = run_json([
        "--target",
        "20240628",
        "--max-digits",
        "100",
        "--backend",
        "cpu-single",
        "--verify",
        "--json",
    ]);

    assert_eq!(value["verification_status"], "passed");
}

#[test]
fn cpu_multi_verify_sets_json_status_to_passed() {
    let value = run_json([
        "--target",
        "20240628",
        "--max-digits",
        "100",
        "--backend",
        "cpu-multi",
        "--threads",
        "2",
        "--verify",
        "--json",
    ]);

    assert_eq!(value["backend"], "cpu-multi");
    assert_eq!(value["threads"], 2);
    assert_eq!(value["verification_status"], "passed");
}

fn run_json<const N: usize>(args: [&str; N]) -> serde_json::Value {
    let output = Command::cargo_bin("pi-birthday-bench")
        .expect("binary exists")
        .args(args)
        .output()
        .expect("command runs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    serde_json::from_str(&stdout).expect("stdout is JSON")
}
