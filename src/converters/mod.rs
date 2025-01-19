use crate::converters::pyproject_updater::PyprojectUpdater;
use crate::schema::pyproject::DependencyGroupSpecification;
use indexmap::IndexMap;
use log::{error, info, warn};
use owo_colors::OwoColorize;
#[cfg(test)]
use std::any::Any;
use std::fmt::Debug;
use std::format;
use std::fs::{remove_file, File};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use toml_edit::DocumentMut;

pub mod pip;
pub mod pipenv;
pub mod poetry;
mod pyproject_updater;

type DependencyGroupsAndDefaultGroups = (
    Option<IndexMap<String, Vec<DependencyGroupSpecification>>>,
    Option<Vec<String>>,
);

#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct ConverterOptions {
    pub project_path: PathBuf,
    pub dry_run: bool,
    pub skip_lock: bool,
    pub ignore_locked_versions: bool,
    pub keep_old_metadata: bool,
    pub dependency_groups_strategy: DependencyGroupsStrategy,
}

/// Converts a project from a package manager to uv.
pub trait Converter: Debug {
    /// Performs the conversion from the current package manager to uv.
    fn convert_to_uv(&self) {
        let pyproject_path = self.get_project_path().join("pyproject.toml");
        let updated_pyproject_string = self.build_uv_pyproject();

        if self.is_dry_run() {
            info!(
                "{}\n{}",
                "Migrated pyproject.toml:".bold(),
                updated_pyproject_string
            );
            return;
        }

        let mut pyproject_file = File::create(&pyproject_path).unwrap();

        pyproject_file
            .write_all(updated_pyproject_string.as_bytes())
            .unwrap();

        self.delete_migrated_files().unwrap();
        self.lock_dependencies();
        self.remove_constraint_dependencies(updated_pyproject_string);

        info!(
            "{}",
            format!(
                "Successfully migrated project from {} to uv!\n",
                self.get_package_manager_name()
            )
            .bold()
            .green()
        );
    }

    /// Build `pyproject.toml` for uv package manager based on current package manager data.
    fn build_uv_pyproject(&self) -> String;

    /// Name of the current package manager.
    fn get_package_manager_name(&self) -> String;

    /// Get the options chosen by the user to perform the migration, such as the project path,
    /// whether locking should be performed at the end of the migration, ...
    fn get_converter_options(&self) -> &ConverterOptions;

    /// Path to the project to migrate.
    fn get_project_path(&self) -> PathBuf {
        self.get_converter_options().clone().project_path
    }

    /// Whether to perform the migration in dry-run mode, meaning that the changes are printed out
    /// instead of made for real.
    fn is_dry_run(&self) -> bool {
        self.get_converter_options().dry_run
    }

    /// Whether to skip dependencies locking at the end of the migration.
    fn skip_lock(&self) -> bool {
        self.get_converter_options().skip_lock
    }

    /// Whether to keep current package manager data at the end of the migration.
    fn keep_old_metadata(&self) -> bool {
        self.get_converter_options().keep_old_metadata
    }

    /// Whether to keep versions locked in the current package manager (if it supports lock files)
    /// when locking dependencies with uv.
    fn respect_locked_versions(&self) -> bool {
        !self.get_converter_options().ignore_locked_versions
    }

    /// Dependency groups strategy to use when writing development dependencies in dependency
    /// groups.
    fn get_dependency_groups_strategy(&self) -> DependencyGroupsStrategy {
        self.get_converter_options().dependency_groups_strategy
    }

    /// List of files tied to the current package manager to delete at the end of the migration.
    fn get_migrated_files_to_delete(&self) -> Vec<String>;

    /// Delete files tied to the current package manager at the end of the migration, unless user
    /// has chosen to keep the current package manager data.
    fn delete_migrated_files(&self) -> std::io::Result<()> {
        if self.keep_old_metadata() {
            return Ok(());
        }

        for file in self.get_migrated_files_to_delete() {
            let path = self.get_project_path().join(file);

            if path.exists() {
                remove_file(path)?;
            }
        }

        Ok(())
    }

    /// Lock dependencies with uv, unless user has explicitly opted out of locking dependencies.
    fn lock_dependencies(&self) {
        if !self.skip_lock() && lock_dependencies(self.get_project_path().as_ref(), false).is_err()
        {
            warn!(
                "An error occurred when locking dependencies, so \"{}\" was not created.",
                "uv.lock".bold()
            );
        }
    }

    /// Get dependencies constraints to set in `constraint-dependencies` under `[tool.uv]` section,
    /// to keep dependencies locked to the same versions as they are with the current package
    /// manager.
    fn get_constraint_dependencies(&self) -> Option<Vec<String>>;

    /// Remove `constraint-dependencies` from `[tool.uv]` in `pyproject.toml`, unless user has
    /// opted out of keeping versions locked in the current package manager.
    ///
    /// Also lock dependencies, to remove `constraints` from `[manifest]` in lock file, unless user
    /// has opted out of locking dependencies.
    fn remove_constraint_dependencies(&self, updated_pyproject_toml: String) {
        if !self.respect_locked_versions() {
            return;
        }

        let mut pyproject_updater = PyprojectUpdater {
            pyproject: &mut updated_pyproject_toml.parse::<DocumentMut>().unwrap(),
        };
        if let Some(updated_pyproject) = pyproject_updater.remove_constraint_dependencies() {
            let mut pyproject_file =
                File::create(self.get_project_path().join("pyproject.toml")).unwrap();
            pyproject_file
                .write_all(updated_pyproject.to_string().as_bytes())
                .unwrap();

            // Lock dependencies a second time, to remove constraints from lock file.
            if !self.skip_lock()
                && lock_dependencies(self.get_project_path().as_ref(), true).is_err()
            {
                warn!("An error occurred when locking dependencies after removing constraints.");
            }
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any;
}

/// Lock dependencies with uv by running `uv lock` command.
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

#[derive(clap::ValueEnum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DependencyGroupsStrategy {
    SetDefaultGroups,
    IncludeInDev,
    KeepExisting,
    MergeIntoDev,
}
