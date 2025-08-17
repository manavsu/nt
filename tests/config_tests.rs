use std::fs;
use std::path::PathBuf;
use std::env;
use std::sync::{Mutex, MutexGuard};

use nt::config::{
    RuntimeConfig,
    DEFAULT_NOTE_FILE_LITERAL,
    DEFAULT_DATETIME_FORMAT_PATTERN,
    resolve_default_config_file_path,
};

static ENV_MUTEX: Mutex<()> = Mutex::new(());

fn setup_temp_env() -> (tempfile::TempDir, MutexGuard<'static, ()>) {
    let guard = ENV_MUTEX.lock().unwrap();
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let home = tmp.path().join("home");
    let config_base = tmp.path().join("config");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&config_base).unwrap();

    unsafe {
        env::set_var("HOME", &home);
        env::set_var("XDG_CONFIG_HOME", &config_base);
    }

    (tmp, guard)
}

#[test]
fn default_when_file_absent() {
    let (_tmp, _env_guard) = setup_temp_env();
    let cfg = RuntimeConfig::load_or_default().expect("load default");
    assert_eq!(cfg.configured_note_file_literal, DEFAULT_NOTE_FILE_LITERAL);
    assert_eq!(cfg.datetime_format_pattern, DEFAULT_DATETIME_FORMAT_PATTERN);

    let expanded_expected = std::env::var("HOME").unwrap() + "/daybook.txt";
    assert_eq!(cfg.expanded_note_file_path, PathBuf::from(expanded_expected));
}

#[test]
fn persist_writes_only_non_defaults() {
    let (_tmp, _env_guard) = setup_temp_env();
    let cfg = RuntimeConfig::load_or_default().unwrap();
    cfg.persist().unwrap();

    let path = resolve_default_config_file_path().unwrap();
    let contents = fs::read_to_string(path).unwrap();
    assert!(contents.trim().is_empty(), "contents was: {:?}", contents);
}

#[test]
fn persist_and_reload_custom_values() {
    let (_tmp, _env_guard) = setup_temp_env();

    let mut cfg = RuntimeConfig::load_or_default().unwrap();
    cfg.configured_note_file_literal = "~/notes/log.txt".to_string();
    cfg.datetime_format_pattern = "%Y-%m-%d".to_string();
    cfg.persist().unwrap();

    let reloaded = RuntimeConfig::load_or_default().unwrap();
    assert_eq!(reloaded.configured_note_file_literal, "~/notes/log.txt");
    assert_eq!(reloaded.datetime_format_pattern, "%Y-%m-%d");

    let home = std::env::var("HOME").unwrap();
    assert_eq!(reloaded.expanded_note_file_path, PathBuf::from(format!("{home}/notes/log.txt")));
}

#[test]
fn tilde_not_expanded_if_not_leading_slash_pattern() {
    let (_tmp, _env_guard) = setup_temp_env();

    let config_path = resolve_default_config_file_path().unwrap();
    if let Some(parent) = config_path.parent() { fs::create_dir_all(parent).unwrap(); }
    fs::write(&config_path, "note_file = \"~someone/alt.txt\"\n").unwrap();

    let cfg = RuntimeConfig::load_or_default().unwrap();
    assert_eq!(cfg.configured_note_file_literal, "~someone/alt.txt");
    assert_eq!(cfg.expanded_note_file_path, PathBuf::from("~someone/alt.txt"));
}

#[test]
fn creates_parent_directory_for_note_file() {
    let (_tmp, _env_guard) = setup_temp_env();

    let mut cfg = RuntimeConfig::load_or_default().unwrap();
    cfg.configured_note_file_literal = "~/deep/nested/notes.txt".into();
    cfg.persist().unwrap();

    let reloaded = RuntimeConfig::load_or_default().unwrap();
    assert!(reloaded.expanded_note_file_path.parent().unwrap().exists());
}

#[test]
fn modifying_only_one_field_results_in_partial_toml() {
    let (_tmp, _env_guard) = setup_temp_env();

    let mut cfg = RuntimeConfig::load_or_default().unwrap();
    cfg.datetime_format_pattern = "%Y".into();
    cfg.persist().unwrap();

    let path = resolve_default_config_file_path().unwrap();
    let contents = fs::read_to_string(path).unwrap();
    assert!(contents.contains("datetime_format = \"%Y\""));
    assert!(!contents.contains("note_file ="));
}
