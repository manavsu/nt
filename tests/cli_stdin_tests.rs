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
fn append_from_stdin_multiline_creates_two_separate_notes() {
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

    // Now print last lines. Expect at least two lines and both entries separately present.
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
    assert!(lines.len() >= 2, "expected at least 2 lines: {stdout}");
    // Ensure they are not on the same line
    let first_matches: Vec<&str> = lines.iter().copied().filter(|l| l.contains("first line")).collect();
    let second_matches: Vec<&str> = lines.iter().copied().filter(|l| l.contains("second line")).collect();
    assert_eq!(first_matches.len(), 1, "expected one line containing first line: {stdout}");
    assert_eq!(second_matches.len(), 1, "expected one line containing second line: {stdout}");
    assert_ne!(first_matches[0], second_matches[0], "expected different lines for each note");
}

#[test]
fn append_from_stdin_whitespace_only_errors() { // unchanged behavior
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

#[test]
fn append_from_stdin_reports_added_count() {
    let tmp_dir = TempDir::new().unwrap();
    let note_file_path = tmp_dir.path().join("notes.txt");
    let (_cfg_handle, cfg_path) = temp_config(&note_file_path);

    let mut child = Command::new(env!("CARGO_BIN_EXE_nt"))
        .arg("--config-path")
        .arg(&cfg_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn nt");
    {
        let stdin = child.stdin.as_mut().unwrap();
        write!(stdin, "line a\nline b\n\nline c\n  \n").unwrap(); // includes blank/whitespace lines
    }
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "expected success");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("added 3 notes"), "stdout missing plural summary: {stdout}");

    let print_out = Command::new(env!("CARGO_BIN_EXE_nt"))
        .arg("--config-path")
        .arg(&cfg_path)
        .arg("--print")
        .arg("10")
        .output()
        .expect("print run");
    assert!(print_out.status.success());
    let printed = String::from_utf8(print_out.stdout).unwrap();
    assert!(printed.contains("line a"));
    assert!(printed.contains("line b"));
    assert!(printed.contains("line c"));
}
