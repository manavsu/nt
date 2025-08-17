use std::io::{Cursor};

use nt::notes::{append_note, tail_lines};

#[test]
fn append_note_writes_single_line_with_trailing_newline() {
    let mut buf = Vec::new();
    append_note(&mut buf, "12:00", "Lunch time").unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert_eq!(s, "12:00 Lunch time\n");
}

#[test]
fn tail_lines_returns_last_n_in_order() {
    let data = b"a\nb\nc\nd\n";
    let cursor = Cursor::new(data);
    let lines = tail_lines(cursor, 2).unwrap();
    assert_eq!(lines, vec!["c", "d"]);
}

#[test]
fn tail_lines_zero_count_returns_empty() {
    let data = b"a\nb\n";
    let cursor = Cursor::new(data);
    let lines = tail_lines(cursor, 0).unwrap();
    assert!(lines.is_empty());
}

#[test]
fn tail_lines_handles_fewer_lines_than_requested() {
    let data = b"only\n";
    let cursor = Cursor::new(data);
    let lines = tail_lines(cursor, 5).unwrap();
    assert_eq!(lines, vec!["only"]);
}
