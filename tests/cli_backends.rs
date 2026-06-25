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
    assert!(stdout.contains("- cpu-multi: available"));
    assert!(stdout.contains("- cuda-compute: unavailable, build with --features cuda"));
    assert!(stdout.contains("- cuda-search-only: unavailable, build with --features cuda"));
    assert!(stdout.contains("- hip: unavailable, build with --features hip"));
    assert!(stdout.contains("- opencl: unavailable, build with --features opencl"));
    assert!(stdout.contains("- vulkan: unavailable, build with --features vulkan"));
    assert!(!stdout.contains("phase="));
    assert!(!stdout.contains("target:"));
}

#[test]
fn gpu_backend_stub_returns_clear_error() {
    for (backend, feature) in [
        ("cuda-compute", "cuda"),
        ("cuda-search-only", "cuda"),
        ("hip", "hip"),
        ("opencl", "opencl"),
        ("vulkan", "vulkan"),
    ] {
        let output = Command::cargo_bin("pi-birthday-bench")
            .expect("binary exists")
            .args([
                "--target",
                "20240628",
                "--max-digits",
                "100",
                "--backend",
                backend,
                "--no-progress",
            ])
            .output()
            .expect("command runs");

        assert!(!output.status.success());

        let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
        assert!(stderr.contains(&format!(
            "backend '{backend}' is not available in this build"
        )));
        assert!(stderr.contains(&format!("hint: rebuild with --features {feature}")));
    }
}
