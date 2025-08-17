use nt::config::RuntimeConfig;

fn main() {
    let cfg = RuntimeConfig::load_or_default().expect("failed to load config");
    println!("Note file: {}", cfg.expanded_note_file_path.display());
}
