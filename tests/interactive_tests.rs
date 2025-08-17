use std::fs;
use std::io::Cursor;
use tempfile::TempDir;

use nt::interactive::{run_interactive_session, InteractiveOutcome};
use nt::time::Clock;
use nt::time::SystemClock; // not used directly but ensure module linkage

// Helper FixedClock sequence for tests
struct SeqClock {
    times: Vec<String>,
    idx: std::cell::Cell<usize>,
}
impl SeqClock { fn new(times: Vec<&str>) -> Self { Self { times: times.into_iter().map(|s| s.to_string()).collect(), idx: std::cell::Cell::new(0) } } }
impl Clock for SeqClock {
    fn now_formatted(&self, _pattern: &str) -> String {
        let i = self.idx.get();
        let v = if i < self.times.len() { &self.times[i] } else { self.times.last().unwrap() };
        self.idx.set(i+1);
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
fn interactive_adds_each_nonblank_line_with_timestamp_and_preserves_trailing_spaces() {
    let tmp_dir = TempDir::new().unwrap();
    let note_file = tmp_dir.path().join("notes.txt");
    // Simulate three lines, with one blank and trailing spaces on second
    let input = b"first line\n   \nsecond line  \n"; // trailing two spaces
    let mut cursor = Cursor::new(&input[..]);
    let clock = SeqClock::new(vec!["T1", "T2"]);
    let outcome = run_interactive_session(&mut cursor, Vec::new(), false, &clock, "%Y-%m-%d %H:%M", &note_file).unwrap();
    match outcome { InteractiveOutcome::Added(n) => assert_eq!(n, 2), _ => panic!("expected Added") }
    let contents = fs::read_to_string(&note_file).unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "T1 first line");
    assert_eq!(lines[1], "T2 second line  "); // trailing spaces preserved
}

#[test]
fn interactive_ignores_only_blank_lines_and_returns_empty() {
    let tmp_dir = TempDir::new().unwrap();
    let note_file = tmp_dir.path().join("notes.txt");
    let input = b"   \n\n\t\n"; // all whitespace
    let mut cursor = Cursor::new(&input[..]);
    let clock = SeqClock::new(vec!["T1"]);
    let outcome = run_interactive_session(&mut cursor, Vec::new(), false, &clock, "%Y-%m-%d %H:%M", &note_file).unwrap();
    match outcome { InteractiveOutcome::Empty => {}, _ => panic!("expected Empty") }
    assert!(!note_file.exists(), "note file should not be created when empty");
}

#[test]
fn interactive_immediate_eof_is_empty() {
    let tmp_dir = TempDir::new().unwrap();
    let note_file = tmp_dir.path().join("notes.txt");
    let input = b""; // immediate EOF
    let mut cursor = Cursor::new(&input[..]);
    let clock = SeqClock::new(vec!["T1"]);
    let outcome = run_interactive_session(&mut cursor, Vec::new(), false, &clock, "%Y-%m-%d %H:%M", &note_file).unwrap();
    match outcome { InteractiveOutcome::Empty => {}, _ => panic!("expected Empty") }
    assert!(!note_file.exists(), "note file should not be created when empty");
}
