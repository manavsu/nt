use std::{process::Command, fs, path::PathBuf};
use tempfile::TempDir;

// Integration test: run binary pointing at a config whose note file does not exist.
#[test]
fn print_when_file_missing_shows_friendly_message() {
    let tmp = TempDir::new().expect("tempdir");
    let note_file_path = tmp.path().join("notes.txt");
    let config_path = tmp.path().join("cfg.toml");
    let config_contents = format!("note_file = \"{}\"\n", note_file_path.display());
    fs::write(&config_path, config_contents).unwrap();

    assert!(!note_file_path.exists(), "note file should not exist yet");

    let output = Command::new(env!("CARGO_BIN_EXE_nt"))
        .arg("--config-path")
        .arg(config_path.as_os_str())
        .arg("--print")
        .output()
        .expect("failed to run nt binary");

    assert!(output.status.success(), "expected success exit, got {:?}", output.status);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("no notes have been made"), "stdout was: {stdout}");
}
