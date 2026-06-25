use assert_cmd::Command;

#[test]
fn list_backends_does_not_require_target_or_max_digits() {
    let output = Command::cargo_bin("pi-birthday-bench")
        .expect("binary exists")
        .arg("--list-backends")
        .output()
        .expect("command runs");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");

    assert!(stdout.contains("available backends:"));
    assert!(stdout.contains("- cpu-single: available"));
    assert!(stdout.contains("- cpu-multi: unavailable, not implemented"));
    assert!(stdout.contains("- cuda-compute: unavailable, build with --features cuda"));
    assert!(stdout.contains("- cuda-search-only: unavailable, build with --features cuda"));
    assert!(stdout.contains("- hip: unavailable, not implemented"));
    assert!(stdout.contains("- opencl: unavailable, not implemented"));
    assert!(stdout.contains("- vulkan: unavailable, not implemented"));
    assert!(!stdout.contains("phase="));
    assert!(!stdout.contains("target:"));
}
