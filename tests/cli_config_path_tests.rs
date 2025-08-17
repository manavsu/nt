use std::process::Command;

#[test]
fn config_path_flag_prints_default_path_and_exits() {
    let output = Command::new(env!("CARGO_BIN_EXE_nt"))
        .arg("--config-path")
        .output()
        .expect("run nt");
    assert!(
        output.status.success(),
        "expected success: {:?}",
        output.status
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let line = stdout.trim();
    assert!(!line.is_empty(), "expected non-empty path");
    assert!(
        line.ends_with("nt.toml"),
        "expected path to end with nt.toml, got: {line}"
    );
    // Should not print extra lines
    assert_eq!(
        stdout.lines().count(),
        1,
        "expected exactly one line of output: {stdout}"
    );
}
