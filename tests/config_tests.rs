use std::path::PathBuf;

use nt::config::{
    DEFAULT_DATETIME_FORMAT_PATTERN, DEFAULT_NOTE_FILE_LITERAL, RuntimeConfig,
    serialize_diff_from_default,
};

fn fake_home() -> PathBuf {
    PathBuf::from("/home/testuser")
}

// Helper to construct a RuntimeConfig from a TOML snippet by manually parsing just the supported fields.
fn build_runtime_config_from_test_toml_manual_parse(toml_str: &str) -> RuntimeConfig {
    let mut note_file_literal: Option<String> = None;
    let mut datetime_format: Option<String> = None;
    for line in toml_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("note_file = ") {
            note_file_literal = Some(rest.trim_matches('"').to_string());
            continue;
        }
        if let Some(rest) = line.strip_prefix("datetime_format = ") {
            datetime_format = Some(rest.trim_matches('"').to_string());
            continue;
        }
    }
    let home = fake_home();
    nt::config::RuntimeConfig::from_parts(
        note_file_literal.unwrap_or_else(|| DEFAULT_NOTE_FILE_LITERAL.to_string()),
        datetime_format.unwrap_or_else(|| DEFAULT_DATETIME_FORMAT_PATTERN.to_string()),
        &home,
        false,
    )
    .unwrap()
}

#[test]
fn creates_default_runtime_config_when_toml_is_empty() {
    let cfg = build_runtime_config_from_test_toml_manual_parse("");
    assert_eq!(cfg.configured_note_file_literal, DEFAULT_NOTE_FILE_LITERAL);
    assert_eq!(cfg.datetime_format_pattern, DEFAULT_DATETIME_FORMAT_PATTERN);
    assert_eq!(
        cfg.expanded_note_file_path,
        PathBuf::from("/home/testuser/daybook.txt")
    );
}

#[test]
fn parses_custom_note_file_and_datetime_format_and_expands_tilde() {
    let toml = r#"note_file = "~/notes/log.txt"
datetime_format = "%Y-%m-%d""#;
    let cfg = build_runtime_config_from_test_toml_manual_parse(toml);
    assert_eq!(cfg.configured_note_file_literal, "~/notes/log.txt");
    assert_eq!(cfg.datetime_format_pattern, "%Y-%m-%d");
    assert_eq!(
        cfg.expanded_note_file_path,
        PathBuf::from("/home/testuser/notes/log.txt")
    );
}

#[test]
fn does_not_expand_tilde_when_followed_by_username_segment() {
    let toml = r#"note_file = "~other/alt.txt""#;
    let cfg = build_runtime_config_from_test_toml_manual_parse(toml);
    assert_eq!(cfg.configured_note_file_literal, "~other/alt.txt");
    assert_eq!(cfg.expanded_note_file_path, PathBuf::from("~other/alt.txt"));
}

#[test]
fn serializes_only_non_default_fields_to_toml() {
    let cfg = RuntimeConfig {
        configured_note_file_literal: DEFAULT_NOTE_FILE_LITERAL.to_string(),
        expanded_note_file_path: PathBuf::from("/home/testuser/daybook.txt"),
        datetime_format_pattern: "%Y".to_string(),
    };
    let toml = serialize_diff_from_default(&cfg).unwrap();
    assert!(toml.contains("datetime_format = \"%Y\""));
    assert!(!toml.contains("note_file"));
}

#[test]
fn round_trips_config_by_serializing_and_reparsing() {
    let initial = RuntimeConfig {
        configured_note_file_literal: "~/n.txt".into(),
        expanded_note_file_path: PathBuf::from("/home/testuser/n.txt"),
        datetime_format_pattern: "%Y-%m".into(),
    };
    let toml = serialize_diff_from_default(&initial).unwrap();
    let reparsed = build_runtime_config_from_test_toml_manual_parse(&toml);
    assert_eq!(
        reparsed.configured_note_file_literal,
        initial.configured_note_file_literal
    );
    assert_eq!(
        reparsed.datetime_format_pattern,
        initial.datetime_format_pattern
    );
}
