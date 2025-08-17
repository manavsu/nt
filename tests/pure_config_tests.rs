use std::path::PathBuf;

use nt::config::{
    parse_toml_without_fs,
    serialize_runtime_config_diff,
    RuntimeConfig,
    DEFAULT_NOTE_FILE_LITERAL,
    DEFAULT_DATETIME_FORMAT_PATTERN,
};

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
