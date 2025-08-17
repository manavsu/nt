use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DEFAULT_NOTE_FILE_LITERAL: &str = "~/daybook.txt";
pub const DEFAULT_DATETIME_FORMAT_PATTERN: &str = "%H:%M - %-m/%-d/%y";
pub const CONFIG_FILE_NAME: &str = "nt.toml";

#[derive(Debug, Error)]
pub enum ConfigLoadSaveError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("toml parse error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),
    #[error("toml serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("unable to locate user home directory")]
    MissingHomeDirectory,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct TomlConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    note_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    datetime_format: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub configured_note_file_literal: String,
    pub expanded_note_file_path: PathBuf,
    pub datetime_format_pattern: String,
}

pub fn resolve_default_config_directory() -> Option<PathBuf> {
    dirs::config_dir().map(|base| base.join("nt"))
}

pub fn resolve_default_config_file_path() -> Result<PathBuf, ConfigLoadSaveError> {
    Ok(resolve_default_config_directory()
        .ok_or(ConfigLoadSaveError::MissingHomeDirectory)?
        .join(CONFIG_FILE_NAME))
}

fn expand_leading_tilde_literal(path_literal: &str, home_directory: &Path) -> PathBuf {
    if let Some(remainder) = path_literal.strip_prefix("~/") {
        home_directory.join(remainder)
    } else {
        PathBuf::from(path_literal)
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        let home_directory = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let expanded_default_note_file =
            expand_leading_tilde_literal(DEFAULT_NOTE_FILE_LITERAL, &home_directory);
        Self {
            configured_note_file_literal: DEFAULT_NOTE_FILE_LITERAL.to_string(),
            expanded_note_file_path: expanded_default_note_file,
            datetime_format_pattern: DEFAULT_DATETIME_FORMAT_PATTERN.to_string(),
        }
    }
}

impl RuntimeConfig {
    pub fn load_or_default() -> Result<Self, ConfigLoadSaveError> {
        let config_file_path = resolve_default_config_file_path()?;
        if !config_file_path.exists() {
            return Ok(RuntimeConfig::default());
        }
        let mut file_contents = String::new();
        fs::File::open(config_file_path)?.read_to_string(&mut file_contents)?;
        let parsed: TomlConfig = toml::from_str(&file_contents)?;
        Self::from_parsed_toml(parsed)
    }

    pub fn persist(&self) -> Result<(), ConfigLoadSaveError> {
        let destination_path = resolve_default_config_file_path()?;
        if let Some(parent_directory) = destination_path.parent() {
            fs::create_dir_all(parent_directory)?;
        }
        let mut toml_config = TomlConfig::default();
        if self.configured_note_file_literal != DEFAULT_NOTE_FILE_LITERAL {
            toml_config.note_file = Some(self.configured_note_file_literal.clone());
        }
        if self.datetime_format_pattern != DEFAULT_DATETIME_FORMAT_PATTERN {
            toml_config.datetime_format = Some(self.datetime_format_pattern.clone());
        }
        let serialized_toml = toml::to_string_pretty(&toml_config)?;
        fs::write(destination_path, serialized_toml)?;
        Ok(())
    }

    fn from_parsed_toml(parsed: TomlConfig) -> Result<Self, ConfigLoadSaveError> {
        let home_directory = dirs::home_dir().ok_or(ConfigLoadSaveError::MissingHomeDirectory)?;
        let configured_note_file_literal = parsed
            .note_file
            .unwrap_or_else(|| DEFAULT_NOTE_FILE_LITERAL.to_string());
        let expanded_note_file_path =
            expand_leading_tilde_literal(&configured_note_file_literal, &home_directory);
        let datetime_format_pattern = parsed
            .datetime_format
            .unwrap_or_else(|| DEFAULT_DATETIME_FORMAT_PATTERN.to_string());
        if let Some(parent_directory) = expanded_note_file_path.parent() {
            fs::create_dir_all(parent_directory)?;
        }
        Ok(Self {
            configured_note_file_literal,
            expanded_note_file_path,
            datetime_format_pattern,
        })
    }
}
