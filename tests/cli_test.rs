use std::process::Command;

#[test]
fn test_help_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("gwatch"));
    assert!(stdout.contains("--path"));
    assert!(stdout.contains("--verbose"));
    assert!(stdout.contains("--help"));
    assert!(stdout.contains("--version"));
}

#[test]
fn test_version_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("gwatch"));
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_invalid_path_error() {
    let output = Command::new("cargo")
        .args(["run", "--", "--path", "/nonexistent/path/xyz123"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not exist") || output.status.code() == Some(1));
}
