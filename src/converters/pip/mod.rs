mod dependencies;

use crate::converters::pip::dependencies::get_constraint_dependencies;
use crate::converters::pyproject_updater::PyprojectUpdater;
use crate::converters::DependencyGroupsStrategy;
use crate::converters::{lock_dependencies, Converter};
use crate::schema::pep_621::Project;
use crate::schema::pyproject::DependencyGroupSpecification;
use crate::schema::uv::Uv;
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
pub struct Pip {
    pub project_path: PathBuf,
    pub requirements_files: Vec<String>,
    pub dev_requirements_files: Vec<String>,
    pub is_pip_tools: bool,
}

impl Converter for Pip {
    fn convert_to_uv(
        &self,
        dry_run: bool,
        skip_lock: bool,
        ignore_locked_versions: bool,
        keep_old_metadata: bool,
        _dependency_groups_strategy: DependencyGroupsStrategy,
    ) {
        let pyproject_path = self.project_path.join("pyproject.toml");
        let updated_pyproject_string = self.perform_migration(ignore_locked_versions);

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
                self.delete_requirements_files().unwrap();
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

            // There are no locked dependencies for pip, so we only have to remove constraints for
            // pip-tools.
            if self.is_pip_tools && !ignore_locked_versions {
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
                format!(
                    "Successfully migrated project from {} to uv!\n",
                    if self.is_pip_tools {
                        "pip-tools"
                    } else {
                        "pip"
                    }
                )
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

impl Pip {
    fn perform_migration(&self, ignore_locked_versions: bool) -> String {
        let dev_dependencies =
            dependencies::get(&self.project_path, self.dev_requirements_files.clone());

        let dependency_groups = dev_dependencies.map_or_else(
            || None,
            |dependencies| {
                let mut groups = IndexMap::new();

                groups.insert(
                    "dev".to_string(),
                    dependencies
                        .iter()
                        .map(|dep| DependencyGroupSpecification::String(dep.to_string()))
                        .collect(),
                );

                Some(groups)
            },
        );

        let project = Project {
            // "name" is required by uv.
            name: Some(String::new()),
            // "version" is required by uv.
            version: Some("0.0.1".to_string()),
            dependencies: dependencies::get(&self.project_path, self.requirements_files.clone()),
            ..Default::default()
        };

        let uv = Uv {
            package: Some(false),
            constraint_dependencies: get_constraint_dependencies(
                ignore_locked_versions,
                self.is_pip_tools,
                &self.project_path,
                self.requirements_files.clone(),
                self.dev_requirements_files.clone(),
            ),
            ..Default::default()
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

    fn delete_requirements_files(&self) -> std::io::Result<()> {
        for requirements_file in self
            .requirements_files
            .iter()
            .chain(&self.dev_requirements_files)
        {
            let requirements_path = self.project_path.join(requirements_file);

            if requirements_path.exists() {
                remove_file(requirements_path.clone())?;
            }
        }

        // For pip-tools, also delete `.txt` files generated from `.in` files.
        if self.is_pip_tools {
            for requirements_file in self
                .requirements_files
                .iter()
                .chain(&self.dev_requirements_files)
                .map(|file| file.replace(".in", ".txt"))
            {
                let requirements_path = self.project_path.join(requirements_file);

                if requirements_path.exists() {
                    remove_file(requirements_path.clone())?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perform_pip_tools_migration() {
        let pip_tools = Pip {
            project_path: PathBuf::from("tests/fixtures/pip_tools/full"),
            requirements_files: vec!["requirements.in".to_string()],
            dev_requirements_files: vec!["requirements-dev.in".to_string()],
            is_pip_tools: true,
        };

        insta::assert_toml_snapshot!(pip_tools.perform_migration(true));
    }

    #[test]
    fn test_perform_pip_tools_all_files_migration() {
        let pip_tools = Pip {
            project_path: PathBuf::from("tests/fixtures/pip_tools/full"),
            requirements_files: vec!["requirements.in".to_string()],
            dev_requirements_files: vec![
                "requirements-dev.in".to_string(),
                "requirements-typing.in".to_string(),
            ],
            is_pip_tools: true,
        };

        insta::assert_toml_snapshot!(pip_tools.perform_migration(true));
    }

    #[test]
    fn test_perform_pip_tools_with_lock_file() {
        let pip_tools = Pip {
            project_path: PathBuf::from("tests/fixtures/pip_tools/with_lock_file"),
            requirements_files: vec!["requirements.in".to_string()],
            dev_requirements_files: vec![
                "requirements-dev.in".to_string(),
                "requirements-typing.in".to_string(),
            ],
            is_pip_tools: true,
        };

        insta::assert_toml_snapshot!(pip_tools.perform_migration(false));
    }

    #[test]
    fn test_perform_pip_migration() {
        let pip = Pip {
            project_path: PathBuf::from("tests/fixtures/pip/full"),
            requirements_files: vec!["requirements.txt".to_string()],
            dev_requirements_files: vec!["requirements-dev.txt".to_string()],
            is_pip_tools: false,
        };

        insta::assert_toml_snapshot!(pip.perform_migration(true));
    }

    #[test]
    fn test_perform_pip_all_files_migration() {
        let pip = Pip {
            project_path: PathBuf::from("tests/fixtures/pip/full"),
            requirements_files: vec!["requirements.txt".to_string()],
            dev_requirements_files: vec![
                "requirements-dev.txt".to_string(),
                "requirements-typing.txt".to_string(),
            ],
            is_pip_tools: false,
        };

        insta::assert_toml_snapshot!(pip.perform_migration(true));
    }
}
