use std::path::PathBuf;

use nt::config::{
    DEFAULT_DATETIME_FORMAT_PATTERN, DEFAULT_NOTE_FILE_LITERAL, RuntimeConfig,
    serialize_diff_from_default, RuntimeConfig as _, // placeholder to prevent unused warnings
};

fn fake_home() -> PathBuf { PathBuf::from("/home/testuser") }

// Helper to parse TOML using public pieces (simulate direct parsing logic)
fn parse_via_from_parts(toml_str: &str) -> RuntimeConfig {
    // Manual minimal parse replicating fields we support; this avoids adding a public parse API.
    let mut note_file_literal: Option<String> = None;
    let mut datetime_format: Option<String> = None;
    for line in toml_str.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if let Some(rest) = line.strip_prefix("note_file = ") { note_file_literal = Some(rest.trim_matches('"').to_string()); continue; }
        if let Some(rest) = line.strip_prefix("datetime_format = ") { datetime_format = Some(rest.trim_matches('"').to_string()); continue; }
    }
    let home = fake_home();
    nt::config::RuntimeConfig::from_parts(
        note_file_literal.unwrap_or_else(|| DEFAULT_NOTE_FILE_LITERAL.to_string()),
        datetime_format.unwrap_or_else(|| DEFAULT_DATETIME_FORMAT_PATTERN.to_string()),
        &home,
        false,
    ).unwrap()
}

#[test]
fn parse_empty_toml_defaults() {
    let cfg = parse_via_from_parts("");
    assert_eq!(cfg.configured_note_file_literal, DEFAULT_NOTE_FILE_LITERAL);
    assert_eq!(cfg.datetime_format_pattern, DEFAULT_DATETIME_FORMAT_PATTERN);
    assert_eq!(cfg.expanded_note_file_path, PathBuf::from("/home/testuser/daybook.txt"));
}

#[test]
fn parse_custom_values() {
    let toml = r#"note_file = "~/notes/log.txt"
datetime_format = "%Y-%m-%d""#;
    let cfg = parse_via_from_parts(toml);
    assert_eq!(cfg.configured_note_file_literal, "~/notes/log.txt");
    assert_eq!(cfg.datetime_format_pattern, "%Y-%m-%d");
    assert_eq!(cfg.expanded_note_file_path, PathBuf::from("/home/testuser/notes/log.txt"));
}

#[test]
fn tilde_not_expanded_for_user_form() {
    let toml = r#"note_file = "~other/alt.txt""#;
    let cfg = parse_via_from_parts(toml);
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
    let toml = serialize_diff_from_default(&cfg).unwrap();
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
    let toml = serialize_diff_from_default(&initial).unwrap();
    let reparsed = parse_via_from_parts(&toml);
    assert_eq!(reparsed.configured_note_file_literal, initial.configured_note_file_literal);
    assert_eq!(reparsed.datetime_format_pattern, initial.datetime_format_pattern);
}

