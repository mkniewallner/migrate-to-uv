mod build_backend;
mod dependencies;
mod project;
mod sources;
pub mod version;

use crate::converters::poetry::build_backend::get_hatch;
use crate::converters::poetry::dependencies::get_constraint_dependencies;
use crate::converters::pyproject_updater::PyprojectUpdater;
use crate::converters::DependencyGroupsStrategy;
use crate::converters::{lock_dependencies, Converter};
use crate::schema::pep_621::Project;
use crate::schema::pyproject::PyProject;
use crate::schema::uv::{SourceContainer, Uv};
use crate::toml::PyprojectPrettyFormatter;
use indexmap::IndexMap;
use log::{info, warn};
use owo_colors::OwoColorize;
#[cfg(test)]
use std::any::Any;
use std::fs;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;
use toml_edit::visit_mut::VisitMut;
use toml_edit::DocumentMut;

const FILES_TO_DELETE: &[&str] = &["poetry.lock", "poetry.toml"];

#[derive(Debug, PartialEq, Eq)]
pub struct Poetry {
    pub project_path: PathBuf,
}

impl Converter for Poetry {
    fn convert_to_uv(
        &self,
        dry_run: bool,
        skip_lock: bool,
        ignore_locked_versions: bool,
        keep_old_metadata: bool,
        dependency_groups_strategy: DependencyGroupsStrategy,
    ) {
        let pyproject_path = self.project_path.join("pyproject.toml");
        let updated_pyproject_string = self.perform_migration(
            ignore_locked_versions,
            keep_old_metadata,
            dependency_groups_strategy,
        );

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
            return;
        }

        let mut pyproject_file = File::create(&pyproject_path).unwrap();

        pyproject_file
            .write_all(updated_pyproject_string.as_bytes())
            .unwrap();

        if !keep_old_metadata {
            self.delete_poetry_references().unwrap();
        }

        if !dry_run && !skip_lock && lock_dependencies(self.project_path.as_ref(), false).is_err() {
            warn!(
                "An error occurred when locking dependencies, so \"{}\" was not created.",
                "uv.lock".bold()
            );
        }

        if !ignore_locked_versions {
            let mut pyproject_updater = PyprojectUpdater {
                pyproject: &mut updated_pyproject_string.parse::<DocumentMut>().unwrap(),
            };
            if let Some(updated_pyproject) = pyproject_updater.remove_constraint_dependencies() {
                let mut pyproject_file = File::create(pyproject_path).unwrap();
                pyproject_file
                    .write_all(updated_pyproject.to_string().as_bytes())
                    .unwrap();

                // Lock dependencies a second time, to remove constraints from lock file.
                if !dry_run
                    && !skip_lock
                    && lock_dependencies(self.project_path.as_ref(), true).is_err()
                {
                    warn!(
                        "An error occurred when locking dependencies after removing constraints."
                    );
                }
            }
        }

        info!(
            "{}",
            "Successfully migrated project from Poetry to uv!\n"
                .bold()
                .green()
        );
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Poetry {
    fn perform_migration(
        &self,
        ignore_locked_versions: bool,
        keep_old_metadata: bool,
        dependency_groups_strategy: DependencyGroupsStrategy,
    ) -> String {
        let pyproject_toml_content =
            fs::read_to_string(self.project_path.join("pyproject.toml")).unwrap_or_default();
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

        let optional_dependencies =
            dependencies::get_optional(&mut poetry_dependencies, poetry.extras);

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
            constraint_dependencies: get_constraint_dependencies(
                ignore_locked_versions,
                &self.project_path.join("poetry.lock"),
            ),
        };

        let hatch = get_hatch(
            poetry.packages.as_ref(),
            poetry.include.as_ref(),
            poetry.exclude.as_ref(),
        );

        let mut updated_pyproject = pyproject_toml_content.parse::<DocumentMut>().unwrap();
        let mut pyproject_updater = PyprojectUpdater {
            pyproject: &mut updated_pyproject,
        };

        pyproject_updater.insert_build_system(
            build_backend::get_new_build_system(pyproject.build_system).as_ref(),
        );
        pyproject_updater.insert_pep_621(&project);
        pyproject_updater.insert_dependency_groups(dependency_groups.as_ref());
        pyproject_updater.insert_uv(&uv);
        pyproject_updater.insert_hatch(hatch.as_ref());

        if !keep_old_metadata {
            remove_pyproject_poetry_section(&mut updated_pyproject);
        }

        let mut visitor = PyprojectPrettyFormatter {
            parent_keys: Vec::new(),
        };
        visitor.visit_document_mut(&mut updated_pyproject);

        updated_pyproject.to_string()
    }

    fn delete_poetry_references(&self) -> std::io::Result<()> {
        for file in FILES_TO_DELETE {
            let path = self.project_path.join(file);

            if path.exists() {
                remove_file(path)?;
            }
        }

        Ok(())
    }
}

fn remove_pyproject_poetry_section(pyproject: &mut DocumentMut) {
    pyproject
        .get_mut("tool")
        .unwrap()
        .as_table_mut()
        .unwrap()
        .remove("poetry");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_perform_migration_multiple_readmes() {
        let tmp_dir = tempdir().unwrap();
        let project_path = tmp_dir.path();

        let pyproject_content = r#"
        [tool.poetry]
        readme = ["README1.md", "README2.md"]
        "#;

        let mut pyproject_file = File::create(project_path.join("pyproject.toml")).unwrap();
        pyproject_file
            .write_all(pyproject_content.as_bytes())
            .unwrap();

        let poetry = Poetry {
            project_path: PathBuf::from(project_path),
        };

        insta::assert_snapshot!(poetry.perform_migration(true, false, DependencyGroupsStrategy::SetDefaultGroups), @r###"
        [project]
        name = ""
        version = "0.0.1"
        readme = "README1.md"
        "###);
    }
}
