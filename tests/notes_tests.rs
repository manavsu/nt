use std::io::Cursor;

use nt::notes::{append_note_line_to_writer, collect_last_n_lines_from_reader};

#[test]
fn append_note_line_to_writer_writes_timestamp_space_text_newline() {
    let mut buf = Vec::new();
    append_note_line_to_writer(&mut buf, "12:00", "Lunch time").unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert_eq!(s, "12:00 Lunch time\n");
}

#[test]
fn collect_last_n_lines_from_reader_returns_last_two_lines_in_order() {
    let data = b"a\nb\nc\nd\n";
    let cursor = Cursor::new(data);
    let lines = collect_last_n_lines_from_reader(cursor, 2).unwrap();
    assert_eq!(lines, vec!["c", "d"]);
}

#[test]
fn collect_last_n_lines_from_reader_zero_returns_empty_vec() {
    let data = b"a\nb\n";
    let cursor = Cursor::new(data);
    let lines = collect_last_n_lines_from_reader(cursor, 0).unwrap();
    assert!(lines.is_empty());
}

#[test]
fn collect_last_n_lines_from_reader_with_fewer_available_returns_existing_lines() {
    let data = b"only\n";
    let cursor = Cursor::new(data);
    let lines = collect_last_n_lines_from_reader(cursor, 5).unwrap();
    assert_eq!(lines, vec!["only"]);
}
