use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::notes::append_note_line_to_file_with_clock;
use crate::time::Clock;

pub enum InteractiveOutcome {
    Added(usize),
    Empty,
}

pub fn run_interactive_session<C: Clock, R: BufRead, W: Write>(
    reader: &mut R,
    mut writer: W,
    prompt_enabled: bool,
    clock: &C,
    datetime_pattern: &str,
    note_file_path: &Path,
) -> io::Result<InteractiveOutcome> {
    let mut line_buf = String::new();
    if prompt_enabled {
        writer.write_all(b"> ")?;
        writer.flush()?;
    }
    let bytes = reader.read_line(&mut line_buf)?;
    if bytes == 0 {
        // EOF immediately
        return Ok(InteractiveOutcome::Empty);
    }
    if line_buf.ends_with('\n') {
        line_buf.pop();
    }
    if line_buf.ends_with('\r') {
        line_buf.pop();
    }
    if line_buf.trim().is_empty() {
        return Ok(InteractiveOutcome::Empty);
    }
    append_note_line_to_file_with_clock(note_file_path, clock, datetime_pattern, &line_buf)?;
    Ok(InteractiveOutcome::Added(1))
}
