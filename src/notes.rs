use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use crate::time::Clock;

pub fn append_note_line_to_file(path: &Path, timestamp: &str, text: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    let mut buf_writer = BufWriter::new(file);
    append_note_line_to_writer(&mut buf_writer, timestamp, text)
}

pub fn append_note_line_to_file_with_clock<C: Clock>(
    path: &Path,
    clock: &C,
    pattern: &str,
    text: &str,
) -> io::Result<()> {
    let ts = clock.now_formatted(pattern);
    append_note_line_to_file(path, &ts, text)
}

pub fn collect_last_n_lines_from_file(path: &Path, count: usize) -> io::Result<Vec<String>> {
    let file = OpenOptions::new().read(true).open(path)?;
    let reader = BufReader::new(file);
    collect_last_n_lines_from_reader(reader, count)
}

/// Attempts to collect the last N lines, but returns Ok(None) if the file does not exist.
pub fn collect_last_n_lines_from_file_allow_missing(
    path: &Path,
    count: usize,
) -> io::Result<Option<Vec<String>>> {
    match collect_last_n_lines_from_file(path, count) {
        Ok(lines) => Ok(Some(lines)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn collect_last_n_lines_from_reader<R: BufRead>(
    reader: R,
    count: usize,
) -> io::Result<Vec<String>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let mut deque: VecDeque<String> = VecDeque::with_capacity(count);
    for line_result in reader.lines() {
        let line = line_result?;
        if deque.len() == count {
            deque.pop_front();
        }
        deque.push_back(line);
    }
    Ok(deque.into_iter().collect())
}

pub fn append_note_line_to_writer<W: Write>(
    writer: &mut W,
    timestamp: &str,
    text: &str,
) -> io::Result<()> {
    writer.write_all(timestamp.as_bytes())?;
    writer.write_all(b" ")?;
    writer.write_all(text.as_bytes())?;
    writer.write_all(b"\n")?;
    Ok(())
}
