use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process;

const CONFIG_FILE: &str = "mlg.json";
const CONTENT_DIR: &str = "content";
const DEFAULT_CONFIG: &str = "{}\n";

pub fn run() {
    let cwd = env::current_dir().expect("failed to determine the current directory");

    match init_in(&cwd) {
        Ok(messages) => {
            for message in messages {
                println!("{message}");
            }
        }
        Err(error) => {
            eprintln!("Failed to initialize Mathlingua collection: {error}");
            process::exit(1);
        }
    }
}

fn init_in(root: &Path) -> io::Result<Vec<String>> {
    let mut messages = Vec::new();

    let config_path = root.join(CONFIG_FILE);
    if config_path.exists() {
        messages.push(format!("Skipping {CONFIG_FILE}; it already exists"));
    } else {
        fs::write(&config_path, DEFAULT_CONFIG)?;
        messages.push(format!("Created {CONFIG_FILE}"));
    }

    let content_path = root.join(CONTENT_DIR);
    if content_path.exists() {
        messages.push(format!("Skipping {CONTENT_DIR}/; it already exists"));
    } else {
        fs::create_dir(&content_path)?;
        messages.push(format!("Created {CONTENT_DIR}/"));
    }

    Ok(messages)
}

#[cfg(test)]
mod tests;
