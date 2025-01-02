mod dependencies;
mod project;
mod sources;

use crate::converters::pyproject_updater::PyprojectUpdater;
use crate::converters::Converter;
use crate::converters::DependencyGroupsStrategy;
use crate::schema::pep_621::Project;
use crate::schema::pipenv::Pipfile;
use crate::schema::uv::{SourceContainer, Uv};
use crate::toml::PyprojectPrettyFormatter;
use indexmap::IndexMap;
use log::info;
use owo_colors::OwoColorize;
use std::default::Default;
use std::fs;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use toml_edit::visit_mut::VisitMut;
use toml_edit::DocumentMut;

pub struct Pipenv {
    pub project_path: PathBuf,
}

impl Converter for Pipenv {
    fn convert_to_uv(
        &self,
        dry_run: bool,
        keep_old_metadata: bool,
        dependency_groups_strategy: DependencyGroupsStrategy,
    ) {
        let pipfile_path = self.project_path.join("Pipfile");
        let pyproject_path = self.project_path.join("pyproject.toml");
        let updated_pyproject_string =
            perform_migration(&pipfile_path, &pyproject_path, dependency_groups_strategy);

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
                delete_pipenv_references(&self.project_path).unwrap();
            }

            info!(
                "{}",
                "Successfully migrated project from Pipenv to uv!\n"
                    .bold()
                    .green()
            );
        }
    }
}

fn perform_migration(
    pipfile_path: &Path,
    pyproject_path: &Path,
    dependency_groups_strategy: DependencyGroupsStrategy,
) -> String {
    let pipfile_content = fs::read_to_string(pipfile_path).unwrap();
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

fn delete_pipenv_references(project_path: &Path) -> std::io::Result<()> {
    let pipfile_path = project_path.join("Pipfile");

    if pipfile_path.exists() {
        remove_file(pipfile_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perform_migration() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/pipenv/full/Pipfile"),
            Path::new("tests/fixtures/pipenv/full/pyproject.toml"),
            DependencyGroupsStrategy::SetDefaultGroups,
        ));
    }

    #[test]
    fn test_perform_migration_dep_group_include_in_dev() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/pipenv/full/Pipfile"),
            Path::new("tests/fixtures/pipenv/full/pyproject.toml"),
            DependencyGroupsStrategy::IncludeInDev,
        ));
    }

    #[test]
    fn test_perform_migration_dep_group_keep_existing() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/pipenv/full/Pipfile"),
            Path::new("tests/fixtures/pipenv/full/pyproject.toml"),
            DependencyGroupsStrategy::KeepExisting,
        ));
    }

    #[test]
    fn test_perform_migration_dep_group_merge_in_dev() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/pipenv/full/Pipfile"),
            Path::new("tests/fixtures/pipenv/full/pyproject.toml"),
            DependencyGroupsStrategy::MergeIntoDev,
        ));
    }

    #[test]
    fn test_perform_migration_python_full_version() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/pipenv/python_full_version/Pipfile"),
            Path::new("tests/fixtures/pipenv/python_full_version/pyproject.toml"),
            DependencyGroupsStrategy::SetDefaultGroups,
        ));
    }

    #[test]
    fn test_perform_migration_empty_requires() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/pipenv/empty_requires/Pipfile"),
            Path::new("tests/fixtures/pipenv/empty_requires/pyproject.toml"),
            DependencyGroupsStrategy::SetDefaultGroups,
        ));
    }

    #[test]
    fn test_perform_migration_minimal_pipfile() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/pipenv/minimal/Pipfile"),
            Path::new("tests/fixtures/pipenv/minimal/pyproject.toml"),
            DependencyGroupsStrategy::SetDefaultGroups,
        ));
    }
}
