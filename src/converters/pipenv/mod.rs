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

const FILES_TO_DELETE: &[&str] = &["Pipfile", "Pipfile.lock"];

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
        for file in FILES_TO_DELETE {
            let path = self.project_path.join(file);

            if path.exists() {
                remove_file(path)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_perform_migration_python_full_version() {
        let tmp_dir = tempdir().unwrap();
        let project_path = tmp_dir.path();

        let pipfile_content = r#"
        [requires]
        python_full_version = "3.13.1"
        "#;

        let mut pipfile_file = File::create(project_path.join("Pipfile")).unwrap();
        pipfile_file.write_all(pipfile_content.as_bytes()).unwrap();

        let pipenv = Pipenv {
            project_path: PathBuf::from(project_path),
        };

        insta::assert_snapshot!(pipenv.perform_migration(true, DependencyGroupsStrategy::SetDefaultGroups), @r###"
        [project]
        name = ""
        version = "0.0.1"
        requires-python = "==3.13.1"

        [tool.uv]
        package = false
        "###);
    }

    #[test]
    fn test_perform_migration_empty_requires() {
        let tmp_dir = tempdir().unwrap();
        let project_path = tmp_dir.path();

        let pipfile_content = "[requires]";

        let mut pipfile_file = File::create(project_path.join("Pipfile")).unwrap();
        pipfile_file.write_all(pipfile_content.as_bytes()).unwrap();

        let pipenv = Pipenv {
            project_path: PathBuf::from(project_path),
        };

        insta::assert_snapshot!(pipenv.perform_migration(true, DependencyGroupsStrategy::SetDefaultGroups), @r###"
        [project]
        name = ""
        version = "0.0.1"

        [tool.uv]
        package = false
        "###);
    }
}
