use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// Append a note entry to an arbitrary buffered writer.
/// Pure logic so it can be tested with an in-memory buffer.
/// The entry format is: `<timestamp> <text>\n`.
pub fn append_note<W: Write>(writer: &mut W, timestamp: &str, text: &str) -> io::Result<()> {
    writer.write_all(timestamp.as_bytes())?;
    writer.write_all(b" ")?;
    writer.write_all(text.as_bytes())?;
    writer.write_all(b"\n")?;
    Ok(())
}

/// Wrapper that opens (and creates if needed) the note file and appends an entry.
/// Ensures parent directory exists.
pub fn append_note_to_file(path: &Path, timestamp: &str, text: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() { if !parent.as_os_str().is_empty() { fs::create_dir_all(parent)?; } }
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    let mut buf_writer = BufWriter::new(file);
    append_note(&mut buf_writer, timestamp, text)
}

/// Collect the last `count` lines from any `BufRead` into a Vec in chronological order.
pub fn tail_lines<R: BufRead>(reader: R, count: usize) -> io::Result<Vec<String>> {
    if count == 0 { return Ok(Vec::new()); }
    let mut deque: VecDeque<String> = VecDeque::with_capacity(count);
    for line_result in reader.lines() {
        let line = line_result?;
        if deque.len() == count { deque.pop_front(); }
        deque.push_back(line);
    }
    Ok(deque.into_iter().collect())
}

/// Wrapper that opens the note file for reading and returns the last `count` lines.
pub fn tail_file_lines(path: &Path, count: usize) -> io::Result<Vec<String>> {
    let file = OpenOptions::new().read(true).open(path)?;
    let reader = BufReader::new(file);
    tail_lines(reader, count)
}
