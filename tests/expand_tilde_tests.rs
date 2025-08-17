use std::path::PathBuf;

use nt::config::expand_leading_tilde_literal;

fn home() -> PathBuf { PathBuf::from("/home/testuser") }

#[test]
fn expands_when_literal_starts_with_tilde_slash() {
    let expanded = expand_leading_tilde_literal("~/notes/log.txt", &home());
    assert_eq!(expanded, PathBuf::from("/home/testuser/notes/log.txt"));
}

#[test]
fn expands_root_home_directory_when_literal_is_just_tilde_slash() {
    let expanded = expand_leading_tilde_literal("~/", &home());
    assert_eq!(expanded, PathBuf::from("/home/testuser"));
}

#[test]
fn leaves_user_form_unchanged() {
    let expanded = expand_leading_tilde_literal("~other/file.txt", &home());
    assert_eq!(expanded, PathBuf::from("~other/file.txt"));
}

#[test]
fn leaves_relative_path_unchanged() {
    let expanded = expand_leading_tilde_literal("relative/file.txt", &home());
    assert_eq!(expanded, PathBuf::from("relative/file.txt"));
}

#[test]
fn leaves_absolute_path_unchanged() {
    let expanded = expand_leading_tilde_literal("/var/log/syslog", &home());
    assert_eq!(expanded, PathBuf::from("/var/log/syslog"));
}
