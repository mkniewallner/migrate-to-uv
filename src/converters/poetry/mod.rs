mod build_backend;
mod dependencies;
mod project;
mod sources;
pub mod version;

use crate::converters::poetry::build_backend::warn_unsupported_package_keys;
use crate::converters::pyproject_updater::PyprojectUpdater;
use crate::converters::Converter;
use crate::converters::DependencyGroupsStrategy;
use crate::schema::pep_621::Project;
use crate::schema::pyproject::PyProject;
use crate::schema::uv::{SourceContainer, Uv};
use crate::toml::PyprojectPrettyFormatter;
use indexmap::IndexMap;
use log::info;
use owo_colors::OwoColorize;
use std::fs;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use toml_edit::visit_mut::VisitMut;
use toml_edit::DocumentMut;

pub struct Poetry {
    pub project_path: PathBuf,
}

impl Converter for Poetry {
    fn convert_to_uv(
        &self,
        dry_run: bool,
        keep_old_metadata: bool,
        dependency_groups_strategy: DependencyGroupsStrategy,
    ) {
        let pyproject_path = self.project_path.join("pyproject.toml");
        let updated_pyproject_string = perform_migration(
            &pyproject_path,
            keep_old_metadata,
            dependency_groups_strategy,
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
                delete_poetry_references(&self.project_path).unwrap();
            }

            info!(
                "{}",
                "Successfully migrated project from Poetry to uv!\n"
                    .bold()
                    .green()
            );
        }
    }
}

fn perform_migration(
    pyproject_path: &Path,
    keep_old_metadata: bool,
    dependency_groups_strategy: DependencyGroupsStrategy,
) -> String {
    let pyproject_toml_content = fs::read_to_string(pyproject_path).unwrap_or_default();
    let pyproject: PyProject = toml::from_str(pyproject_toml_content.as_str()).unwrap();

    let poetry = pyproject.tool.unwrap().poetry.unwrap();

    let mut uv_source_index: IndexMap<String, SourceContainer> = IndexMap::new();
    let (dependency_groups, uv_default_groups) =
        dependencies::get_dependency_groups_and_default_groups(
            &poetry,
            &mut uv_source_index,
            dependency_groups_strategy,
        );
    let mut poetry_dependencies = poetry.dependencies;

    let python_specification = poetry_dependencies
        .as_mut()
        .and_then(|dependencies| dependencies.shift_remove("python"));

    let optional_dependencies = dependencies::get_optional(&mut poetry_dependencies, poetry.extras);

    let mut poetry_plugins = poetry.plugins;
    let scripts_from_plugins = poetry_plugins
        .as_mut()
        .and_then(|plugins| plugins.shift_remove("console_scripts"));
    let gui_scripts = poetry_plugins
        .as_mut()
        .and_then(|plugins| plugins.shift_remove("gui_scripts"));

    let project = Project {
        // "name" is required by uv.
        name: Some(poetry.name.unwrap_or_default()),
        // "version" is required by uv.
        version: Some(poetry.version.unwrap_or_else(|| "0.0.1".to_string())),
        description: poetry.description,
        authors: project::get_authors(poetry.authors),
        requires_python: python_specification.map(|p| p.to_pep_508()),
        readme: project::get_readme(poetry.readme),
        license: poetry.license,
        maintainers: project::get_authors(poetry.maintainers),
        keywords: poetry.keywords,
        classifiers: poetry.classifiers,
        dependencies: dependencies::get(poetry_dependencies.as_ref(), &mut uv_source_index),
        optional_dependencies,
        urls: project::get_urls(
            poetry.urls,
            poetry.homepage,
            poetry.repository,
            poetry.documentation,
        ),
        scripts: project::get_scripts(poetry.scripts, scripts_from_plugins),
        gui_scripts,
        entry_points: poetry_plugins,
    };

    let uv = Uv {
        package: poetry.package_mode,
        index: sources::get_indexes(poetry.source),
        sources: if uv_source_index.is_empty() {
            None
        } else {
            Some(uv_source_index)
        },
        default_groups: uv_default_groups,
    };

    warn_unsupported_package_keys(
        poetry.packages.as_ref(),
        poetry.include.as_ref(),
        poetry.exclude.as_ref(),
    );

    let mut updated_pyproject = pyproject_toml_content.parse::<DocumentMut>().unwrap();
    let mut pyproject_updater = PyprojectUpdater {
        pyproject: &mut updated_pyproject,
    };

    pyproject_updater
        .insert_build_system(build_backend::get_new_build_system(pyproject.build_system).as_ref());
    pyproject_updater.insert_pep_621(&project);
    pyproject_updater.insert_dependency_groups(dependency_groups.as_ref());
    pyproject_updater.insert_uv(&uv);

    if !keep_old_metadata {
        remove_pyproject_poetry_section(&mut updated_pyproject);
    }

    let mut visitor = PyprojectPrettyFormatter {
        parent_keys: Vec::new(),
    };
    visitor.visit_document_mut(&mut updated_pyproject);

    updated_pyproject.to_string()
}

fn remove_pyproject_poetry_section(pyproject: &mut DocumentMut) {
    pyproject
        .get_mut("tool")
        .unwrap()
        .as_table_mut()
        .unwrap()
        .remove("poetry");
}

fn delete_poetry_references(project_path: &Path) -> std::io::Result<()> {
    let poetry_lock_path = project_path.join("poetry.lock");

    if poetry_lock_path.exists() {
        remove_file(poetry_lock_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perform_migration() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/poetry/full/pyproject.toml"),
            false,
            DependencyGroupsStrategy::SetDefaultGroups,
        ));
    }

    #[test]
    fn test_perform_migration_keep_old_metadata() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/poetry/full/pyproject.toml"),
            true,
            DependencyGroupsStrategy::SetDefaultGroups,
        ));
    }

    #[test]
    fn test_perform_migration_dep_group_include_in_dev() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/poetry/full/pyproject.toml"),
            false,
            DependencyGroupsStrategy::IncludeInDev,
        ));
    }

    #[test]
    fn test_perform_migration_dep_group_keep_existing() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/poetry/full/pyproject.toml"),
            false,
            DependencyGroupsStrategy::KeepExisting,
        ));
    }

    #[test]
    fn test_perform_migration_dep_group_merge_in_dev() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/poetry/full/pyproject.toml"),
            false,
            DependencyGroupsStrategy::MergeIntoDev,
        ));
    }

    #[test]
    fn test_perform_migration_multiple_readmes() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/poetry/multiple_readmes/pyproject.toml"),
            false,
            DependencyGroupsStrategy::SetDefaultGroups,
        ));
    }

    #[test]
    fn test_perform_migration_minimal_pyproject() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/poetry/minimal/pyproject.toml"),
            false,
            DependencyGroupsStrategy::SetDefaultGroups,
        ));
    }
}
