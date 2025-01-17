mod dependencies;
mod project;
mod sources;

use crate::converters::pipenv::dependencies::get_constraint_dependencies;
use crate::converters::pyproject_updater::PyprojectUpdater;
use crate::converters::DependencyGroupsStrategy;
use crate::converters::{lock_dependencies, Converter};
use crate::schema::pep_621::Project;
use crate::schema::pipenv::Pipfile;
use crate::schema::uv::{SourceContainer, Uv};
use crate::toml::PyprojectPrettyFormatter;
use indexmap::IndexMap;
use log::{info, warn};
use owo_colors::OwoColorize;
#[cfg(test)]
use std::any::Any;
use std::default::Default;
use std::fs;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;
use toml_edit::visit_mut::VisitMut;
use toml_edit::DocumentMut;

#[derive(Debug, PartialEq, Eq)]
pub struct Pipenv {
    pub project_path: PathBuf,
}

impl Converter for Pipenv {
    fn convert_to_uv(
        &self,
        dry_run: bool,
        skip_lock: bool,
        ignore_locked_versions: bool,
        keep_old_metadata: bool,
        dependency_groups_strategy: DependencyGroupsStrategy,
    ) {
        let pyproject_path = self.project_path.join("pyproject.toml");
        let updated_pyproject_string =
            self.perform_migration(ignore_locked_versions, dependency_groups_strategy);

        if dry_run {
            let mut pyproject_updater = PyprojectUpdater {
                pyproject: &mut updated_pyproject_string.parse::<DocumentMut>().unwrap(),
            };
            info!(
                "{}\n{}",
                "Migrated pyproject.toml:".bold(),
                pyproject_updater
                    .remove_constraint_dependencies()
                    .map_or(updated_pyproject_string, ToString::to_string)
            );
        } else {
            let mut pyproject_file = File::create(&pyproject_path).unwrap();

            pyproject_file
                .write_all(updated_pyproject_string.as_bytes())
                .unwrap();

            if !keep_old_metadata {
                self.delete_pipenv_references().unwrap();
            }

            if !dry_run
                && !skip_lock
                && lock_dependencies(self.project_path.as_ref(), false).is_err()
            {
                warn!(
                    "An error occurred when locking dependencies, so \"{}\" was not created.",
                    "uv.lock".bold()
                );
            }

            if !ignore_locked_versions {
                let mut pyproject_updater = PyprojectUpdater {
                    pyproject: &mut updated_pyproject_string.parse::<DocumentMut>().unwrap(),
                };
                if let Some(updated_pyproject) = pyproject_updater.remove_constraint_dependencies()
                {
                    let mut pyproject_file = File::create(pyproject_path).unwrap();
                    pyproject_file
                        .write_all(updated_pyproject.to_string().as_bytes())
                        .unwrap();

                    // Lock dependencies a second time, to remove constraints from lock file.
                    if !dry_run
                        && !skip_lock
                        && lock_dependencies(self.project_path.as_ref(), true).is_err()
                    {
                        warn!("An error occurred when locking dependencies after removing constraints.");
                    }
                }
            }

            info!(
                "{}",
                "Successfully migrated project from Pipenv to uv!\n"
                    .bold()
                    .green()
            );
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Pipenv {
    fn perform_migration(
        &self,
        ignore_locked_versions: bool,
        dependency_groups_strategy: DependencyGroupsStrategy,
    ) -> String {
        let pipfile_content = fs::read_to_string(self.project_path.join("Pipfile")).unwrap();
        let pipfile: Pipfile = toml::from_str(pipfile_content.as_str()).unwrap();

        let mut uv_source_index: IndexMap<String, SourceContainer> = IndexMap::new();
        let (dependency_groups, uv_default_groups) =
            dependencies::get_dependency_groups_and_default_groups(
                &pipfile,
                &mut uv_source_index,
                dependency_groups_strategy,
            );

        let project = Project {
            // "name" is required by uv.
            name: Some(String::new()),
            // "version" is required by uv.
            version: Some("0.0.1".to_string()),
            requires_python: project::get_requires_python(pipfile.requires),
            dependencies: dependencies::get(pipfile.packages.as_ref(), &mut uv_source_index),
            ..Default::default()
        };

        let uv = Uv {
            package: Some(false),
            index: sources::get_indexes(pipfile.source),
            sources: if uv_source_index.is_empty() {
                None
            } else {
                Some(uv_source_index)
            },
            default_groups: uv_default_groups,
            constraint_dependencies: get_constraint_dependencies(
                ignore_locked_versions,
                &self.project_path.join("Pipfile.lock"),
            ),
        };

        let pyproject_toml_content =
            fs::read_to_string(self.project_path.join("pyproject.toml")).unwrap_or_default();
        let mut updated_pyproject = pyproject_toml_content.parse::<DocumentMut>().unwrap();
        let mut pyproject_updater = PyprojectUpdater {
            pyproject: &mut updated_pyproject,
        };

        pyproject_updater.insert_pep_621(&project);
        pyproject_updater.insert_dependency_groups(dependency_groups.as_ref());
        pyproject_updater.insert_uv(&uv);

        let mut visitor = PyprojectPrettyFormatter {
            parent_keys: Vec::new(),
        };
        visitor.visit_document_mut(&mut updated_pyproject);

        updated_pyproject.to_string()
    }

    fn delete_pipenv_references(&self) -> std::io::Result<()> {
        let pipfile_path = self.project_path.join("Pipfile");

        if pipfile_path.exists() {
            remove_file(pipfile_path)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perform_migration() {
        let pipenv = Pipenv {
            project_path: PathBuf::from("tests/fixtures/pipenv/full"),
        };

        insta::assert_toml_snapshot!(
            pipenv.perform_migration(true, DependencyGroupsStrategy::SetDefaultGroups)
        );
    }

    #[test]
    fn test_perform_migration_dep_group_include_in_dev() {
        let pipenv = Pipenv {
            project_path: PathBuf::from("tests/fixtures/pipenv/full"),
        };

        insta::assert_toml_snapshot!(
            pipenv.perform_migration(true, DependencyGroupsStrategy::IncludeInDev)
        );
    }

    #[test]
    fn test_perform_migration_dep_group_keep_existing() {
        let pipenv = Pipenv {
            project_path: PathBuf::from("tests/fixtures/pipenv/full"),
        };

        insta::assert_toml_snapshot!(
            pipenv.perform_migration(true, DependencyGroupsStrategy::KeepExisting)
        );
    }

    #[test]
    fn test_perform_migration_dep_group_merge_in_dev() {
        let pipenv = Pipenv {
            project_path: PathBuf::from("tests/fixtures/pipenv/full"),
        };

        insta::assert_toml_snapshot!(
            pipenv.perform_migration(true, DependencyGroupsStrategy::MergeIntoDev)
        );
    }

    #[test]
    fn test_perform_migration_python_full_version() {
        let pipenv = Pipenv {
            project_path: PathBuf::from("tests/fixtures/pipenv/python_full_version"),
        };

        insta::assert_toml_snapshot!(
            pipenv.perform_migration(true, DependencyGroupsStrategy::SetDefaultGroups)
        );
    }

    #[test]
    fn test_perform_migration_empty_requires() {
        let pipenv = Pipenv {
            project_path: PathBuf::from("tests/fixtures/pipenv/empty_requires"),
        };

        insta::assert_toml_snapshot!(
            pipenv.perform_migration(true, DependencyGroupsStrategy::SetDefaultGroups)
        );
    }

    #[test]
    fn test_perform_migration_minimal_pipfile() {
        let pipenv = Pipenv {
            project_path: PathBuf::from("tests/fixtures/pipenv/minimal"),
        };

        insta::assert_toml_snapshot!(
            pipenv.perform_migration(true, DependencyGroupsStrategy::SetDefaultGroups)
        );
    }

    #[test]
    fn test_perform_migration_with_lock_file() {
        let pipenv = Pipenv {
            project_path: PathBuf::from("tests/fixtures/pipenv/with_lock_file"),
        };

        insta::assert_toml_snapshot!(
            pipenv.perform_migration(false, DependencyGroupsStrategy::SetDefaultGroups)
        );
    }
}
