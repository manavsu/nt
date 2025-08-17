use std::fs;
use std::io::Cursor;
use tempfile::TempDir;

use nt::interactive::{InteractiveOutcome, run_interactive_session};
use nt::time::Clock;

struct SeqClock {
    times: Vec<String>,
    idx: std::cell::Cell<usize>,
}
impl SeqClock {
    fn new(times: Vec<&str>) -> Self {
        Self {
            times: times.into_iter().map(|s| s.to_string()).collect(),
            idx: std::cell::Cell::new(0),
        }
    }
}
impl Clock for SeqClock {
    fn now_formatted(&self, _pattern: &str) -> String {
        let i = self.idx.get();
        let v = if i < self.times.len() {
            &self.times[i]
        } else {
            self.times.last().unwrap()
        };
        self.idx.set(i + 1);
        v.clone()
    }
}

fn temp_config_file(note_file: &std::path::Path) -> (tempfile::NamedTempFile, std::path::PathBuf) {
    let cfg = tempfile::NamedTempFile::new().unwrap();
    let path = cfg.path().to_path_buf();
    let contents = format!("note_file = \"{}\"\n", note_file.display());
    fs::write(&path, contents).unwrap();
    (cfg, path)
}

#[test]
fn interactive_single_line_adds_one_note_and_stops() {
    let tmp_dir = TempDir::new().unwrap();
    let note_file = tmp_dir.path().join("notes.txt");
    let input = b"single line entry  \nsecond should be ignored\n"; // second line ignored due to single-line mode
    let mut cursor = Cursor::new(&input[..]);
    let clock = SeqClock::new(vec!["T1"]);
    let outcome = run_interactive_session(
        &mut cursor,
        Vec::new(),
        false,
        &clock,
        "%Y-%m-%d %H:%M",
        &note_file,
    )
    .unwrap();
    match outcome {
        InteractiveOutcome::Added(n) => assert_eq!(n, 1),
        _ => panic!("expected Added(1)"),
    }
    let contents = fs::read_to_string(&note_file).unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "T1 single line entry  "); // trailing spaces preserved
}

#[test]
fn interactive_blank_line_returns_empty() {
    let tmp_dir = TempDir::new().unwrap();
    let note_file = tmp_dir.path().join("notes.txt");
    let input = b"   \nrest ignored"; // whitespace first line => empty
    let mut cursor = Cursor::new(&input[..]);
    let clock = SeqClock::new(vec!["T1"]);
    let outcome = run_interactive_session(
        &mut cursor,
        Vec::new(),
        false,
        &clock,
        "%Y-%m-%d %H:%M",
        &note_file,
    )
    .unwrap();
    match outcome {
        InteractiveOutcome::Empty => {}
        _ => panic!("expected Empty"),
    }
    assert!(
        !note_file.exists(),
        "note file should not be created when empty"
    );
}

#[test]
fn interactive_immediate_eof_is_empty() {
    let tmp_dir = TempDir::new().unwrap();
    let note_file = tmp_dir.path().join("notes.txt");
    let input = b""; // immediate EOF
    let mut cursor = Cursor::new(&input[..]);
    let clock = SeqClock::new(vec!["T1"]);
    let outcome = run_interactive_session(
        &mut cursor,
        Vec::new(),
        false,
        &clock,
        "%Y-%m-%d %H:%M",
        &note_file,
    )
    .unwrap();
    match outcome {
        InteractiveOutcome::Empty => {}
        _ => panic!("expected Empty"),
    }
    assert!(
        !note_file.exists(),
        "note file should not be created when empty"
    );
}
