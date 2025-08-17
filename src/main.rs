use nt::cli::{Cli, CommandAction};
use nt::config::RuntimeConfig;
use nt::notes::append_note_line_to_file_with_clock;
use nt::time::SystemClock;
use nt::interactive::{run_interactive_session, InteractiveOutcome};
use std::io::Read;

fn main() {
    let parsed = Cli::parse_action();
    let (cli, action) = match parsed {
        Ok(v) => v,
        Err(e) => {
            let _ = e.print();
            std::process::exit(2);
        }
    };
    let cfg = if let Some(custom) = cli.config_path.as_ref() {
        match RuntimeConfig::load_from_path(custom) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("config load error: {e}");
                std::process::exit(1);
            }
        }
    } else {
        match RuntimeConfig::load_or_default() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("config load error: {e}");
                std::process::exit(1);
            }
        }
    };

    match action {
        CommandAction::Append { text } => {
            let clock = SystemClock;
            if let Err(e) = append_note_line_to_file_with_clock(
                &cfg.expanded_note_file_path,
                &clock,
                &cfg.datetime_format_pattern,
                &text,
            ) {
                eprintln!("write error: {e}");
                std::process::exit(1);
            }
        }
        CommandAction::Print { count } => {
            match nt::notes::collect_last_n_lines_from_file_allow_missing(
                &cfg.expanded_note_file_path,
                count,
            ) {
                Ok(Some(lines)) => {
                    for l in lines {
                        println!("{l}");
                    }
                }
                Ok(None) => {
                    println!("no notes have been made");
                }
                Err(e) => {
                    eprintln!("read error: {e}");
                    std::process::exit(1);
                }
            }
        }
        CommandAction::AppendFromStdin => {
            let mut buf = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
                eprintln!("stdin read error: {e}");
                std::process::exit(1);
            }
            let text_raw = buf.trim_end_matches(['\n', '\r'].as_ref());
            if text_raw.trim().is_empty() {
                eprintln!("note text cannot be empty");
                std::process::exit(2);
            }
            let clock = SystemClock;
            if let Err(e) = append_note_line_to_file_with_clock(
                &cfg.expanded_note_file_path,
                &clock,
                &cfg.datetime_format_pattern,
                text_raw,
            ) {
                eprintln!("write error: {e}");
                std::process::exit(1);
            }
        }
        CommandAction::InteractiveAppend => {
            use std::io::{stdin, stdout, BufReader, IsTerminal};
            let clock = SystemClock;
            let mut reader = BufReader::new(stdin());
            let prompt_enabled = stdout().is_terminal();
            match run_interactive_session(
                &mut reader,
                stdout(),
                prompt_enabled,
                &clock,
                &cfg.datetime_format_pattern,
                &cfg.expanded_note_file_path,
            ) {
                Ok(InteractiveOutcome::Added(n)) => {
                    println!("added {n} note{}", if n == 1 { "" } else { "s" });
                }
                Ok(InteractiveOutcome::Empty) => {
                    eprintln!("no note text provided");
                    std::process::exit(2);
                }
                Err(e) => {
                    eprintln!("interactive error: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}
