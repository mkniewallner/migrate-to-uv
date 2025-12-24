use log::error;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::process;
use which::which;

const UV_EXECUTABLE: &str = "uv";

/// Get the path to uv executable, if it exists.
pub fn get_executable() -> Option<PathBuf> {
    which(UV_EXECUTABLE).ok()
}

/// Ensure that uv executable exists in the PATH, or abort the migration if not found.
pub fn ensure_executable_exists() {
    if get_executable().is_some() {
        return;
    }

    error!("uv executable not found, but it is needed to lock dependencies during migration.");
    error!(
        "Either make sure that uv is installed and in your PATH, or pass \"{}\" to skip locking.",
        "--skip-lock".bold(),
    );
    process::exit(1);
}
