mod dependencies;

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
use std::path::{Path, PathBuf};
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
        keep_old_metadata: bool,
        _dependency_groups_strategy: DependencyGroupsStrategy,
    ) {
        let pyproject_path = self.project_path.join("pyproject.toml");
        let updated_pyproject_string = perform_migration(
            &self.project_path,
            self.requirements_files.clone(),
            self.dev_requirements_files.clone(),
            &pyproject_path,
        );

        if dry_run {
            info!(
                "{}\n{}",
                "Migrated pyproject.toml:".bold(),
                updated_pyproject_string
            );
        } else {
            let mut pyproject_file = File::create(&pyproject_path).unwrap();

            pyproject_file
                .write_all(updated_pyproject_string.as_bytes())
                .unwrap();

            if !keep_old_metadata {
                delete_requirements_files(&self.project_path, &self.requirements_files).unwrap();
                delete_requirements_files(&self.project_path, &self.dev_requirements_files)
                    .unwrap();

                if self.is_pip_tools {
                    delete_requirements_files(
                        &self.project_path,
                        &self
                            .requirements_files
                            .iter()
                            .map(|file| file.replace(".in", ".txt"))
                            .collect(),
                    )
                    .unwrap();
                    delete_requirements_files(
                        &self.project_path,
                        &self
                            .dev_requirements_files
                            .iter()
                            .map(|file| file.replace(".in", ".txt"))
                            .collect(),
                    )
                    .unwrap();
                }
            }

            if !dry_run && !skip_lock && lock_dependencies(self.project_path.as_ref()).is_err() {
                warn!(
                    "Project migrated from {} to uv, but an error occurred when locking dependencies.",
                    if self.is_pip_tools {
                        "pip-tools"
                    } else {
                        "pip"
                    }
                );
                return;
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

fn perform_migration(
    project_path: &Path,
    requirements_files: Vec<String>,
    dev_requirements_files: Vec<String>,
    pyproject_path: &Path,
) -> String {
    let dev_dependencies = dependencies::get(project_path, dev_requirements_files);

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
        dependencies: dependencies::get(project_path, requirements_files),
        ..Default::default()
    };

    let uv = Uv {
        package: Some(false),
        ..Default::default()
    };

    let pyproject_toml_content = fs::read_to_string(pyproject_path).unwrap_or_default();
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

fn delete_requirements_files(
    project_path: &Path,
    requirements_files: &Vec<String>,
) -> std::io::Result<()> {
    for requirements_file in requirements_files {
        let requirements_path = project_path.join(requirements_file);

        if requirements_path.exists() {
            remove_file(requirements_path.clone())?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perform_pip_tools_migration() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/pip_tools/full"),
            vec!["requirements.in".to_string()],
            vec!["requirements-dev.in".to_string()],
            Path::new("tests/fixtures/pip_tools/full/pyproject.toml"),
        ));
    }

    #[test]
    fn test_perform_pip_tools_all_files_migration() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/pip_tools/full"),
            vec!["requirements.in".to_string()],
            vec![
                "requirements-dev.in".to_string(),
                "requirements-typing.in".to_string()
            ],
            Path::new("tests/fixtures/pip_tools/full/pyproject.toml"),
        ));
    }

    #[test]
    fn test_perform_pip_migration() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/pip/full"),
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            Path::new("tests/fixtures/pip/full/pyproject.toml"),
        ));
    }

    #[test]
    fn test_perform_pip_all_files_migration() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/pip/full"),
            vec!["requirements.txt".to_string()],
            vec![
                "requirements-dev.txt".to_string(),
                "requirements-typing.txt".to_string()
            ],
            Path::new("tests/fixtures/pip/full/pyproject.toml"),
        ));
    }
}
