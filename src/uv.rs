use log::{error, info};
use owo_colors::OwoColorize;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;
use which::which;

const UV_EXECUTABLE: &str = "uv";

pub enum LockType {
    ConstraintsRemoval,
    LockWithConstraints,
    LockWithoutConstraints,
}

impl Display for LockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConstraintsRemoval => write!(
                f,
                "Locking dependencies again using \"{}\" to remove constraints...",
                format!("{UV_EXECUTABLE} lock").bold(),
            ),
            Self::LockWithConstraints => write!(
                f,
                "Locking dependencies with constraints from existing lock file(s) using \"{}\"...",
                format!("{UV_EXECUTABLE} lock").bold(),
            ),
            Self::LockWithoutConstraints => write!(
                f,
                "Locking dependencies using \"{}\"...",
                format!("{UV_EXECUTABLE} lock").bold(),
            ),
        }
    }
}

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

/// Lock dependencies with uv by running `uv lock` command.
pub fn lock_dependencies(project_path: &Path, lock_type: &LockType) -> Result<(), ()> {
    // Should be a safe `unwrap`, as we already check at the beginning of the migration if uv is
    // present if we need to invoke it during the migration.
    let uv = get_executable().unwrap();

    info!("{lock_type}");

    Command::new(uv)
        .arg("lock")
        .current_dir(project_path)
        .spawn()
        .map_or_else(
            |e| {
                error!("{e}");
                Err(())
            },
            |lock| match lock.wait_with_output() {
                Ok(output) => {
                    if output.status.success() {
                        Ok(())
                    } else {
                        Err(())
                    }
                }
                Err(e) => {
                    error!("{e}");
                    Err(())
                }
            },
        )
}

/// Get the current version of uv, if uv is found.
pub fn get_version() -> Option<String> {
    let uv = get_executable()?;

    match Command::new(uv)
        .arg("self")
        .arg("version")
        .arg("--short")
        .arg("--no-color")
        .output()
    {
        Ok(output) => {
            String::from_utf8(output.stdout).map_or(None, |stdout| {
                // On some platforms (e.g., Homebrew), some additional information can be displayed,
                // so we need to trim that information.
                if let Some((version, _)) = stdout.split_once(char::is_whitespace) {
                    Some(version.to_string())
                } else {
                    Some(stdout)
                }
            })
        }
        Err(_) => None,
    }
}
