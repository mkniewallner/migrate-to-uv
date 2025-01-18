mod build_backend;
mod dependencies;
mod project;
mod sources;
pub mod version;

use crate::converters::poetry::build_backend::get_hatch;
use crate::converters::poetry::dependencies::get_constraint_dependencies;
use crate::converters::pyproject_updater::PyprojectUpdater;
use crate::converters::Converter;
use crate::converters::ConverterOptions;
use crate::schema::pep_621::Project;
use crate::schema::pyproject::PyProject;
use crate::schema::uv::{SourceContainer, Uv};
use crate::toml::PyprojectPrettyFormatter;
use indexmap::IndexMap;
#[cfg(test)]
use std::any::Any;
use std::fs;
use toml_edit::visit_mut::VisitMut;
use toml_edit::DocumentMut;

#[derive(Debug, PartialEq, Eq)]
pub struct Poetry {
    pub converter_options: ConverterOptions,
}

impl Converter for Poetry {
    fn perform_migration(&self) -> String {
        let pyproject_toml_content =
            fs::read_to_string(self.get_project_path().join("pyproject.toml")).unwrap_or_default();
        let pyproject: PyProject = toml::from_str(pyproject_toml_content.as_str()).unwrap();

        let poetry = pyproject.tool.unwrap().poetry.unwrap();

        let mut uv_source_index: IndexMap<String, SourceContainer> = IndexMap::new();
        let (dependency_groups, uv_default_groups) =
            dependencies::get_dependency_groups_and_default_groups(
                &poetry,
                &mut uv_source_index,
                self.get_dependency_groups_strategy(),
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
                !self.respect_locked_versions(),
                &self.get_project_path().join("poetry.lock"),
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

        if !self.keep_old_metadata() {
            remove_pyproject_poetry_section(&mut updated_pyproject);
        }

        let mut visitor = PyprojectPrettyFormatter {
            parent_keys: Vec::new(),
        };
        visitor.visit_document_mut(&mut updated_pyproject);

        updated_pyproject.to_string()
    }

    fn get_package_manager_name(&self) -> String {
        "Poetry".to_string()
    }

    fn get_converter_options(&self) -> &ConverterOptions {
        &self.converter_options
    }

    fn get_migrated_files_to_delete(&self) -> Vec<String> {
        vec!["poetry.lock".to_string(), "poetry.toml".to_string()]
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
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
    use crate::converters::DependencyGroupsStrategy;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
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
            converter_options: ConverterOptions {
                project_path: PathBuf::from(project_path),
                dry_run: true,
                skip_lock: true,
                ignore_locked_versions: true,
                keep_old_metadata: false,
                dependency_groups_strategy: DependencyGroupsStrategy::SetDefaultGroups,
            },
        };

        insta::assert_snapshot!(poetry.perform_migration(), @r###"
        [project]
        name = ""
        version = "0.0.1"
        readme = "README1.md"
        "###);
    }
}
