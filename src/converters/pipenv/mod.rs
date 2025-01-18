mod dependencies;
mod project;
mod sources;

use crate::converters::pipenv::dependencies::get_constraint_dependencies;
use crate::converters::pyproject_updater::PyprojectUpdater;
use crate::converters::Converter;
use crate::converters::ConverterOptions;
use crate::schema::pep_621::Project;
use crate::schema::pipenv::Pipfile;
use crate::schema::uv::{SourceContainer, Uv};
use crate::toml::PyprojectPrettyFormatter;
use indexmap::IndexMap;
#[cfg(test)]
use std::any::Any;
use std::default::Default;
use std::fs;
use toml_edit::visit_mut::VisitMut;
use toml_edit::DocumentMut;

#[derive(Debug, PartialEq, Eq)]
pub struct Pipenv {
    pub converter_options: ConverterOptions,
}

impl Converter for Pipenv {
    fn perform_migration(&self) -> String {
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
            constraint_dependencies: get_constraint_dependencies(
                !self.respect_locked_versions(),
                &self.get_project_path().join("Pipfile.lock"),
            ),
        };

        let pyproject_toml_content =
            fs::read_to_string(self.get_project_path().join("pyproject.toml")).unwrap_or_default();
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

    fn get_package_manager_name(&self) -> String {
        "Pipenv".to_string()
    }

    fn get_converter_options(&self) -> &ConverterOptions {
        &self.converter_options
    }

    fn get_migrated_files_to_delete(&self) -> Vec<String> {
        vec!["Pipfile".to_string(), "Pipfile.lock".to_string()]
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
                ignore_locked_versions: true,
                keep_old_metadata: false,
                dependency_groups_strategy: DependencyGroupsStrategy::SetDefaultGroups,
            },
        };

        insta::assert_snapshot!(pipenv.perform_migration(), @r###"
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
                ignore_locked_versions: true,
                keep_old_metadata: false,
                dependency_groups_strategy: DependencyGroupsStrategy::SetDefaultGroups,
            },
        };

        insta::assert_snapshot!(pipenv.perform_migration(), @r###"
        [project]
        name = ""
        version = "0.0.1"

        [tool.uv]
        package = false
        "###);
    }
}
