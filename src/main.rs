use nt::cli::{Cli, CommandAction};
use nt::config::RuntimeConfig;
use nt::interactive::{InteractiveOutcome, run_interactive_session};
use nt::notes::append_note_line_to_file_with_clock;
use nt::time::SystemClock;
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
            } else {
                println!("added 1 note");
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
            use std::io::{BufRead, BufReader};
            let stdin = std::io::stdin();
            let reader = BufReader::new(stdin.lock());
            let clock = SystemClock;
            let mut added = 0usize;
            for line_result in reader.lines() {
                match line_result {
                    Ok(mut line) => {
                        // Strip any trailing carriage return (Windows pipes) but preserve other trailing spaces
                        if line.ends_with('\r') { line.pop(); }
                        if line.trim().is_empty() { continue; }
                        if let Err(e) = append_note_line_to_file_with_clock(
                            &cfg.expanded_note_file_path,
                            &clock,
                            &cfg.datetime_format_pattern,
                            &line,
                        ) {
                            eprintln!("write error: {e}");
                            std::process::exit(1);
                        }
                        added += 1;
                    }
                    Err(e) => {
                        eprintln!("stdin read error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            if added == 0 {
                eprintln!("note text cannot be empty");
                std::process::exit(2);
            } else {
                println!("added {added} note{}", if added == 1 { "" } else { "s" });
            }
        }
        CommandAction::InteractiveAppend => {
            use std::io::{BufReader, IsTerminal, stdin, stdout};
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
