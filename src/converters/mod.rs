use crate::schema::pyproject::DependencyGroupSpecification;
use indexmap::IndexMap;
use log::{error, info, warn};
use owo_colors::OwoColorize;
#[cfg(test)]
use std::any::Any;
use std::fmt::Debug;
use std::io::ErrorKind;
use std::path::Path;
use std::process::{Command, Stdio};

pub mod pip;
pub mod pipenv;
pub mod poetry;
mod pyproject_updater;

type DependencyGroupsAndDefaultGroups = (
    Option<IndexMap<String, Vec<DependencyGroupSpecification>>>,
    Option<Vec<String>>,
);

/// Converts a project from a package manager to uv.
pub trait Converter: Debug {
    #[allow(clippy::fn_params_excessive_bools)]
    fn convert_to_uv(
        &self,
        dry_run: bool,
        skip_lock: bool,
        ignore_locked_versions: bool,
        keep_old_metadata: bool,
        dependency_groups_strategy: DependencyGroupsStrategy,
    );

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any;
}

pub fn lock_dependencies(project_path: &Path, is_removing_constraints: bool) -> Result<(), ()> {
    const UV_EXECUTABLE: &str = "uv";

    match Command::new(UV_EXECUTABLE)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(_) => {
            info!(
                "Locking dependencies with \"{}\"{}...",
                format!("{UV_EXECUTABLE} lock").bold(),
                if is_removing_constraints {
                    " again to remove constraints"
                } else {
                    ""
                }
            );

            Command::new(UV_EXECUTABLE)
                .arg("lock")
                .current_dir(project_path)
                .spawn()
                .map_or_else(
                    |_| {
                        error!(
                            "Could not invoke \"{}\" command.",
                            format!("{UV_EXECUTABLE} lock").bold()
                        );
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
        Err(e) if e.kind() == ErrorKind::NotFound => {
            warn!(
                "Could not find \"{}\" executable, skipping locking dependencies.",
                UV_EXECUTABLE.bold()
            );
            Ok(())
        }
        Err(e) => {
            error!("{e}");
            Err(())
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum DependencyGroupsStrategy {
    SetDefaultGroups,
    IncludeInDev,
    KeepExisting,
    MergeIntoDev,
}
