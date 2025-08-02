use clap::Parser;
use std::{path::PathBuf, fs::OpenOptions};
use walkdir::{DirEntry, WalkDir};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    path: PathBuf,
}

fn is_syre_dir(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with(".syre"))
        .unwrap_or(false)
}

fn main() {
    let cli = Cli::parse();
    for entry in WalkDir::new(cli.path)
        .into_iter()
        .filter_entry(|e| !is_syre_dir(e))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Ok(file) = OpenOptions::new().write(true).truncate(true).open(entry.path()) {
            let _ = file.set_len(0);
        }
    }
}
