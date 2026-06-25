use assert_cmd::Command;

#[test]
fn cli_progress_reports_context_to_stderr() {
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
            "cpu-multi",
            "--threads",
            "2",
        ])
        .output()
        .expect("command runs");

    assert!(output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("phase=computing_pi"));
    assert!(stderr.contains("phase=searching"));
    assert!(stderr.contains("backend=cpu-multi"));
    assert!(stderr.contains("target=20240628"));
    assert!(stderr.contains("range=1..100"));
    assert!(stderr.contains("digits_computed=100"));
    assert!(stderr.contains("elapsed_seconds="));
    assert!(stderr.contains("digits_per_second="));
    assert!(stderr.contains("chunk=25"));
    assert!(stderr.contains("threads=2"));
}

#[test]
fn no_progress_suppresses_progress_stderr() {
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
            "--no-progress",
        ])
        .output()
        .expect("command runs");

    assert!(output.status.success());
    assert!(
        output.stderr.is_empty(),
        "stderr should be empty when --no-progress is set"
    );
}
