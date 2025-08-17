use clap::Parser;
use nt::cli::Cli;

#[test]
fn parse_print_default_count() {
    let cli = Cli::try_parse_from(["nt", "-p"]).unwrap();
    assert!(cli.print.is_some());
    assert_eq!(cli.print.unwrap(), Some(10));
}

#[test]
fn parse_print_explicit_count() {
    let cli = Cli::try_parse_from(["nt", "--print", "5"]).unwrap();
    assert_eq!(cli.print.unwrap(), Some(5));
}

#[test]
fn parse_append_text() {
    let cli = Cli::try_parse_from(["nt", "hello", "world"]).unwrap();
    assert!(cli.print.is_none());
    assert_eq!(cli.note, vec!["hello", "world"]);
}
