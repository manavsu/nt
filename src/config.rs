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

#[derive(Debug, Clone, PartialEq, Eq)]
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

pub fn expand_leading_tilde_literal(path_literal: &str, home_directory: &Path) -> PathBuf {
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
        let serialized_toml = serialize_runtime_config_diff(self)?;
        fs::write(destination_path, serialized_toml)?;
        Ok(())
    }

    fn from_parsed_toml(parsed: TomlConfig) -> Result<Self, ConfigLoadSaveError> {
        let home_directory = dirs::home_dir().ok_or(ConfigLoadSaveError::MissingHomeDirectory)?;
        Ok(runtime_config_from_parts(
            parsed
                .note_file
                .unwrap_or_else(|| DEFAULT_NOTE_FILE_LITERAL.to_string()),
            parsed
                .datetime_format
                .unwrap_or_else(|| DEFAULT_DATETIME_FORMAT_PATTERN.to_string()),
            &home_directory,
            true,
        )?)
    }
}

pub fn runtime_config_from_parts(
    note_file_literal: String,
    datetime_format_pattern: String,
    home_directory: &Path,
    create_parent_dir: bool,
) -> Result<RuntimeConfig, ConfigLoadSaveError> {
    let expanded_note_file_path = expand_leading_tilde_literal(&note_file_literal, home_directory);
    if create_parent_dir {
        if let Some(parent_directory) = expanded_note_file_path.parent() {
            if !parent_directory.as_os_str().is_empty() {
                fs::create_dir_all(parent_directory)?;
            }
        }
    }
    Ok(RuntimeConfig {
        configured_note_file_literal: note_file_literal,
        expanded_note_file_path,
        datetime_format_pattern,
    })
}

pub fn parse_toml_without_fs(
    toml_str: &str,
    home_directory: &Path,
) -> Result<RuntimeConfig, ConfigLoadSaveError> {
    let parsed: TomlConfig = toml::from_str(toml_str)?;
    runtime_config_from_parts(
        parsed
            .note_file
            .unwrap_or_else(|| DEFAULT_NOTE_FILE_LITERAL.to_string()),
        parsed
            .datetime_format
            .unwrap_or_else(|| DEFAULT_DATETIME_FORMAT_PATTERN.to_string()),
        home_directory,
        false,
    )
}

pub fn serialize_runtime_config_diff(
    cfg: &RuntimeConfig,
) -> Result<String, ConfigLoadSaveError> {
    let mut toml_config = TomlConfig::default();
    if cfg.configured_note_file_literal != DEFAULT_NOTE_FILE_LITERAL {
        toml_config.note_file = Some(cfg.configured_note_file_literal.clone());
    }
    if cfg.datetime_format_pattern != DEFAULT_DATETIME_FORMAT_PATTERN {
        toml_config.datetime_format = Some(cfg.datetime_format_pattern.clone());
    }
    Ok(toml::to_string_pretty(&toml_config)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_home() -> PathBuf { PathBuf::from("/home/testuser") }

    #[test]
    fn parse_empty_toml_defaults() {
        let cfg = parse_toml_without_fs("", &fake_home()).unwrap();
        assert_eq!(cfg.configured_note_file_literal, DEFAULT_NOTE_FILE_LITERAL);
        assert_eq!(cfg.datetime_format_pattern, DEFAULT_DATETIME_FORMAT_PATTERN);
        assert_eq!(cfg.expanded_note_file_path, PathBuf::from("/home/testuser/daybook.txt"));
    }

    #[test]
    fn parse_custom_values() {
        let toml = r#"note_file = "~/notes/log.txt"
datetime_format = "%Y-%m-%d""#;
        let cfg = parse_toml_without_fs(toml, &fake_home()).unwrap();
        assert_eq!(cfg.configured_note_file_literal, "~/notes/log.txt");
        assert_eq!(cfg.datetime_format_pattern, "%Y-%m-%d");
        assert_eq!(cfg.expanded_note_file_path, PathBuf::from("/home/testuser/notes/log.txt"));
    }

    #[test]
    fn tilde_not_expanded_for_user_form() {
        let toml = r#"note_file = "~other/alt.txt""#;
        let cfg = parse_toml_without_fs(toml, &fake_home()).unwrap();
        assert_eq!(cfg.configured_note_file_literal, "~other/alt.txt");
        assert_eq!(cfg.expanded_note_file_path, PathBuf::from("~other/alt.txt"));
    }

    #[test]
    fn serialize_diff_only() {
        let cfg = RuntimeConfig {
            configured_note_file_literal: DEFAULT_NOTE_FILE_LITERAL.to_string(),
            expanded_note_file_path: PathBuf::from("/home/testuser/daybook.txt"),
            datetime_format_pattern: "%Y".to_string(),
        };
        let toml = serialize_runtime_config_diff(&cfg).unwrap();
        assert!(toml.contains("datetime_format = \"%Y\""));
        assert!(!toml.contains("note_file"));
    }

    #[test]
    fn round_trip_non_defaults() {
        let initial = RuntimeConfig {
            configured_note_file_literal: "~/n.txt".into(),
            expanded_note_file_path: PathBuf::from("/home/testuser/n.txt"),
            datetime_format_pattern: "%Y-%m".into(),
        };
        let toml = serialize_runtime_config_diff(&initial).unwrap();
        let reparsed = parse_toml_without_fs(&toml, &fake_home()).unwrap();
        assert_eq!(reparsed.configured_note_file_literal, initial.configured_note_file_literal);
        assert_eq!(reparsed.datetime_format_pattern, initial.datetime_format_pattern);
    }
}
