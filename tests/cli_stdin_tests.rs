use std::io::Write;
use std::process::{Command, Stdio};
use std::{fs};
use tempfile::TempDir;

fn temp_config(note_file: &std::path::Path) -> (tempfile::NamedTempFile, std::path::PathBuf) {
    let cfg = tempfile::NamedTempFile::new().unwrap();
    let path = cfg.path().to_path_buf();
    let contents = format!("note_file = \"{}\"\n", note_file.display());
    fs::write(&path, contents).unwrap();
    (cfg, path)
}

#[test]
fn append_from_stdin_multiline_creates_two_lines_under_one_timestamp() {
    let tmp_dir = TempDir::new().unwrap();
    let note_file_path = tmp_dir.path().join("notes.txt");
    let (_cfg_handle, cfg_path) = temp_config(&note_file_path);

    let mut child = Command::new(env!("CARGO_BIN_EXE_nt"))
        .arg("--config-path")
        .arg(&cfg_path)
        .stdin(Stdio::piped())
        .spawn()
        .expect("spawn nt");
    {
        let stdin = child.stdin.as_mut().unwrap();
        write!(stdin, "first line\nsecond line\n").unwrap();
    }
    let status = child.wait().unwrap();
    assert!(status.success(), "append via stdin failed");

    // Now print last 2 lines. Expect two lines total and second line matches.
    let output = Command::new(env!("CARGO_BIN_EXE_nt"))
        .arg("--config-path")
        .arg(&cfg_path)
        .arg("--print")
        .arg("5")
        .output()
        .expect("print run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.iter().any(|l| l.contains("first line")), "missing first line: {stdout}");
    assert!(lines.iter().any(|l| l.contains("second line")), "missing second line: {stdout}");
}

#[test]
fn append_from_stdin_whitespace_only_errors() {
    let tmp_dir = TempDir::new().unwrap();
    let note_file_path = tmp_dir.path().join("notes.txt");
    let (_cfg_handle, cfg_path) = temp_config(&note_file_path);

    let mut child = Command::new(env!("CARGO_BIN_EXE_nt"))
        .arg("--config-path")
        .arg(&cfg_path)
        .stdin(Stdio::piped())
        .spawn()
        .expect("spawn nt");
    {
        let stdin = child.stdin.as_mut().unwrap();
        write!(stdin, "   \n\n").unwrap();
    }
    let status = child.wait().unwrap();
    assert!(!status.success(), "expected failure for whitespace only");
}
