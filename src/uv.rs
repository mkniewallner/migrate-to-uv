use log::{error, info, warn};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;
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

/// Lock dependencies with uv by running `uv lock` command.
pub fn lock_dependencies(project_path: &Path, is_removing_constraints: bool) -> Result<(), ()> {
    get_executable().map_or_else(
        || {
            warn!(
                "Could not find \"{}\" executable, skipping \"{}\".",
                UV_EXECUTABLE.bold(),
                format!("{UV_EXECUTABLE} lock").bold(),
            );
            Err(())
        },
        |uv| {
            info!(
                "Locking dependencies with \"{}\"{}...",
                format!("{UV_EXECUTABLE} lock").bold(),
                if is_removing_constraints {
                    " again to remove constraints"
                } else {
                    ""
                }
            );

            Command::new(uv)
                .arg("lock")
                .current_dir(project_path)
                .spawn()
                .map_or_else(
                    |_| {
                        warn!(
                            "Could not invoke \"{}\" command, skipping dependencies locking.",
                            format!("{UV_EXECUTABLE} lock").bold()
                        );
                        Err(())
                    },
                    |lock| match lock.wait_with_output() {
                        Ok(output) => {
                            if output.status.success() {
                                Ok(())
                            } else {
                                error!(
                                    "Error while invoking \"{}\" command.",
                                    format!("{UV_EXECUTABLE} lock").bold(),
                                );
                                Err(())
                            }
                        }
                        Err(e) => {
                            error!(
                                "The following error occurred while invoking \"{}\" command:",
                                format!("{UV_EXECUTABLE} lock").bold(),
                            );
                            error!("{e}");
                            Err(())
                        }
                    },
                )
        },
    )
}
