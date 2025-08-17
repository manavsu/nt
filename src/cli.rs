use clap::{ArgAction, Parser, error::ErrorKind};

#[derive(Parser, Debug)]
#[command(name = "nt", about = "Simple timestamped note taker", version, author)]
pub struct Cli {
    #[arg(short='p', long="print", num_args=0..=1, value_name="N", default_missing_value="10")]
    pub print: Option<Option<usize>>,
    #[arg(long = "config-path", help = "print the default config file path and exit", action = ArgAction::SetTrue)]
    pub show_config_path: bool,
    #[arg(short = 'i', long = "interactive", action = ArgAction::SetTrue, help = "enter interactive single-line mode (press Enter to submit)")]
    pub interactive: bool,
    #[arg(value_name = "NOTE", trailing_var_arg = true)]
    pub note: Vec<String>,
}

pub enum CommandAction {
    Append { text: String },
    Print { count: usize },
    AppendFromStdin,
    InteractiveAppend,
    ShowConfigPath,
}

impl Cli {
    pub fn parse_action() -> Result<CommandAction, clap::Error> {
        let cli = Cli::parse();
        // Handle explicit interactive flag first
        if cli.show_config_path {
            if cli.print.is_some() || cli.interactive || !cli.note.is_empty() {
                return Err(clap::Error::raw(
                    ErrorKind::ArgumentConflict,
                    "--config-path cannot be combined with other options or note text",
                ));
            }
            return Ok(CommandAction::ShowConfigPath);
        }
        if cli.interactive {
            if cli.print.is_some() {
                return Err(clap::Error::raw(
                    ErrorKind::ArgumentConflict,
                    "cannot mix --interactive with --print/-p",
                ));
            }
            if !cli.note.is_empty() {
                return Err(clap::Error::raw(
                    ErrorKind::ArgumentConflict,
                    "cannot supply note text with --interactive",
                ));
            }
            return Ok(CommandAction::InteractiveAppend);
        }
        if let Some(opt) = &cli.print {
            if !cli.note.is_empty() {
                return Err(clap::Error::raw(
                    ErrorKind::ArgumentConflict,
                    "cannot mix note text with --print/-p",
                ));
            }
            let count = opt.unwrap_or(10);
            return Ok(CommandAction::Print { count });
        }
        if cli.note.is_empty() {
            // If no note text provided, allow capturing from stdin when stdin is not a TTY.
            use std::io::IsTerminal;
            if std::io::stdin().is_terminal() {
                // Automatic interactive mode (no args, stdin is TTY)
                return Ok(CommandAction::InteractiveAppend);
            } else {
                return Ok(CommandAction::AppendFromStdin);
            }
        }
        let text = cli.note.join(" ").trim().to_string();
        if text.is_empty() {
            return Err(clap::Error::raw(
                ErrorKind::InvalidValue,
                "note text cannot be empty",
            ));
        }
        Ok(CommandAction::Append { text })
    }
}
