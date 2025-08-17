use clap::{Parser, error::ErrorKind};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "nt", about = "Simple timestamped note taker", version, author)]
pub struct Cli {
    #[arg(short='p', long="print", num_args=0..=1, value_name="N", default_missing_value="10")]
    pub print: Option<Option<usize>>,
    #[arg(long = "config-path", value_name = "PATH")]
    pub config_path: Option<PathBuf>,
    #[arg(value_name = "NOTE", trailing_var_arg = true)]
    pub note: Vec<String>,
}

pub enum CommandAction {
    Append { text: String },
    Print { count: usize },
}

impl Cli {
    pub fn parse_action() -> Result<(Self, CommandAction), clap::Error> {
        let cli = Cli::parse();
        if let Some(opt) = &cli.print {
            if !cli.note.is_empty() {
                return Err(clap::Error::raw(
                    ErrorKind::ArgumentConflict,
                    "cannot mix note text with --print/-p",
                ));
            }
            let count = opt.unwrap_or(10);
            return Ok((cli, CommandAction::Print { count }));
        }
        if cli.note.is_empty() {
            return Err(clap::Error::raw(
                ErrorKind::MissingRequiredArgument,
                "supply note text or use --print",
            ));
        }
        let text = cli.note.join(" ").trim().to_string();
        if text.is_empty() {
            return Err(clap::Error::raw(
                ErrorKind::InvalidValue,
                "note text cannot be empty",
            ));
        }
        Ok((cli, CommandAction::Append { text }))
    }
}
