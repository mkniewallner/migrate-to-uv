mod dependencies;
mod project;

use crate::converters::Converter;
use crate::converters::ConverterOptions;
use crate::converters::pyproject_updater::PyprojectUpdater;
use crate::converters::setuptools::project::{get_authors, get_urls, get_version};
use crate::schema::pep_621::{License, Project};
use crate::schema::pyproject::PyProject;
use crate::toml::PyprojectPrettyFormatter;
use configparser::ini::Ini;
use std::fs;
use toml_edit::DocumentMut;
use toml_edit::visit_mut::VisitMut;

#[derive(Debug, PartialEq, Eq)]
pub struct Setuptools {
    pub converter_options: ConverterOptions,
}

impl Converter for Setuptools {
    fn build_uv_pyproject(&self) -> String {
        let pyproject_toml_content =
            fs::read_to_string(self.get_project_path().join("pyproject.toml")).unwrap_or_default();
        let pyproject: PyProject = toml::from_str(pyproject_toml_content.as_str()).unwrap();

        let setup_cfg =
            fs::read_to_string(self.get_project_path().join("setup.cfg")).unwrap_or_default();

        let mut config = Ini::new();
        config.set_multiline(true);
        config.read(setup_cfg).unwrap();

        let config_map = config.get_map().unwrap();

        let project = Project {
            name: config.get("metadata", "name").or(Some(String::new())),
            version: Some(get_version(&config)),
            description: get_with_alias(&config, "metadata", "description", "summary"),
            authors: get_authors(
                config.get("metadata", "author"),
                get_with_alias(&config, "metadata", "author_email", "author-email"),
            ),
            requires_python: config.get("options", "python_requires"),
            license: config.get("metadata", "license").map(License::String),
            maintainers: get_authors(
                config.get("metadata", "maintainer"),
                get_with_alias(&config, "metadata", "maintainer_email", "maintainer-email"),
            ),
            keywords: get_comma_separated_value(config.get("metadata", "keywords")),
            classifiers: get_multi_line_value(config.get("metadata", "classifiers")),
            dependencies: get_multi_line_value(config.get("options", "install_requires")),
            optional_dependencies: dependencies::get_optional(
                config_map["options.extras_require"].clone(),
            ),
            urls: get_urls(config),
            ..Default::default()
        };

        let mut updated_pyproject = pyproject_toml_content.parse::<DocumentMut>().unwrap();
        let mut pyproject_updater = PyprojectUpdater {
            pyproject: &mut updated_pyproject,
        };

        pyproject_updater.insert_pep_621(&self.build_project(pyproject.project, project));

        let mut visitor = PyprojectPrettyFormatter::default();
        visitor.visit_document_mut(&mut updated_pyproject);

        updated_pyproject.to_string()
    }

    fn get_package_manager_name(&self) -> String {
        "Setuptools".to_string()
    }

    fn get_converter_options(&self) -> &ConverterOptions {
        &self.converter_options
    }

    fn get_migrated_files_to_delete(&self) -> Vec<String> {
        vec!["setup.cfg".to_string(), "setup.py".to_string()]
    }

    fn get_constraint_dependencies(&self) -> Option<Vec<String>> {
        None
    }
}

fn get_with_alias(config: &Ini, section: &str, key: &str, alias: &str) -> Option<String> {
    config.get(section, key).or(config.get(section, alias))
}

fn get_comma_separated_value(value: Option<String>) -> Option<Vec<String>> {
    value.map(|v| v.split(',').map(|v| v.trim().to_string()).collect())
}

fn get_multi_line_value(value: Option<String>) -> Option<Vec<String>> {
    value.map(|v| v.trim().lines().map(ToString::to_string).collect())
}
