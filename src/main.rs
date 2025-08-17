use nt::cli::{Cli, CommandAction};
use nt::config::RuntimeConfig;
use nt::notes::{append_note_line_to_file_with_clock, collect_last_n_lines_from_file};
use nt::time::SystemClock;

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
        CommandAction::Print { count } => match collect_last_n_lines_from_file(
            &cfg.expanded_note_file_path,
            count,
        ) {
            Ok(lines) => {
                for l in lines {
                    println!("{l}");
                }
            }
            Err(e) => {
                eprintln!("read error: {e}");
                std::process::exit(1);
            }
        },
    }
}
