mod dependencies;
mod project;
mod sources;

use crate::converters::Converter;
use crate::converters::ConverterOptions;
use crate::converters::pyproject_updater::PyprojectUpdater;
use crate::schema::pep_621::Project;
use crate::schema::pipenv::{PipenvLock, Pipfile};
use crate::schema::pyproject::PyProject;
use crate::schema::uv::{SourceContainer, Uv};
use crate::toml::PyprojectPrettyFormatter;
use indexmap::IndexMap;
use log::warn;
use owo_colors::OwoColorize;
#[cfg(test)]
use std::any::Any;
use std::default::Default;
use std::fs;
use toml_edit::DocumentMut;
use toml_edit::visit_mut::VisitMut;

#[derive(Debug, PartialEq, Eq)]
pub struct Pipenv {
    pub converter_options: ConverterOptions,
}

impl Converter for Pipenv {
    fn build_uv_pyproject(&self) -> String {
        let pyproject_toml_content =
            fs::read_to_string(self.get_project_path().join("pyproject.toml")).unwrap_or_default();
        let pyproject: PyProject = toml::from_str(pyproject_toml_content.as_str()).unwrap();

        let pipfile_content = fs::read_to_string(self.get_project_path().join("Pipfile")).unwrap();
        let pipfile: Pipfile = toml::from_str(pipfile_content.as_str()).unwrap();

        let mut uv_source_index: IndexMap<String, SourceContainer> = IndexMap::new();
        let (dependency_groups, uv_default_groups) =
            dependencies::get_dependency_groups_and_default_groups(
                &pipfile,
                &mut uv_source_index,
                self.get_dependency_groups_strategy(),
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
            constraint_dependencies: self.get_constraint_dependencies(),
        };

        let pyproject_toml_content =
            fs::read_to_string(self.get_project_path().join("pyproject.toml")).unwrap_or_default();
        let mut updated_pyproject = pyproject_toml_content.parse::<DocumentMut>().unwrap();
        let mut pyproject_updater = PyprojectUpdater {
            pyproject: &mut updated_pyproject,
        };

        pyproject_updater.insert_pep_621(&self.build_project(pyproject.project, project));
        pyproject_updater.insert_dependency_groups(dependency_groups.as_ref());
        pyproject_updater.insert_uv(&uv);

        let mut visitor = PyprojectPrettyFormatter::default();
        visitor.visit_document_mut(&mut updated_pyproject);

        updated_pyproject.to_string()
    }

    fn get_package_manager_name(&self) -> String {
        "Pipenv".to_string()
    }

    fn get_converter_options(&self) -> &ConverterOptions {
        &self.converter_options
    }

    fn get_migrated_files_to_delete(&self) -> Vec<String> {
        vec!["Pipfile".to_string(), "Pipfile.lock".to_string()]
    }

    fn get_constraint_dependencies(&self) -> Option<Vec<String>> {
        let pipenv_lock_path = self.get_project_path().join("Pipfile.lock");

        if self.is_dry_run() || !self.respect_locked_versions() || !pipenv_lock_path.exists() {
            return None;
        }

        let pipenv_lock_content = fs::read_to_string(pipenv_lock_path).unwrap();
        let Ok(pipenv_lock) = serde_json::from_str::<PipenvLock>(pipenv_lock_content.as_str())
        else {
            warn!(
                "Could not parse \"{}\", dependencies will not be kept to their current locked versions.",
                "Pipfile.lock".bold()
            );
            return None;
        };

        let constraint_dependencies: Vec<String> = pipenv_lock
            .category_groups
            .unwrap_or_default()
            .values()
            .flatten()
            .map(|(name, spec)| format!("{}{}", name, spec.version))
            .collect();

        if constraint_dependencies.is_empty() {
            None
        } else {
            Some(constraint_dependencies)
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converters::DependencyGroupsStrategy;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
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
            converter_options: ConverterOptions {
                project_path: PathBuf::from(project_path),
                dry_run: true,
                skip_lock: true,
                skip_uv_checks: false,
                ignore_locked_versions: true,
                replace_project_section: false,
                keep_old_metadata: false,
                dependency_groups_strategy: DependencyGroupsStrategy::SetDefaultGroups,
            },
        };

        insta::assert_snapshot!(pipenv.build_uv_pyproject(), @r###"
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
            converter_options: ConverterOptions {
                project_path: PathBuf::from(project_path),
                dry_run: true,
                skip_lock: true,
                skip_uv_checks: false,
                ignore_locked_versions: true,
                replace_project_section: false,
                keep_old_metadata: false,
                dependency_groups_strategy: DependencyGroupsStrategy::SetDefaultGroups,
            },
        };

        insta::assert_snapshot!(pipenv.build_uv_pyproject(), @r###"
        [project]
        name = ""
        version = "0.0.1"

        [tool.uv]
        package = false
        "###);
    }
}
