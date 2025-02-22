mod dependencies;

use crate::converters::Converter;
use crate::converters::ConverterOptions;
use crate::converters::pyproject_updater::PyprojectUpdater;
use crate::schema::pep_621::Project;
use crate::schema::pyproject::{DependencyGroupSpecification, PyProject};
use crate::schema::uv::Uv;
use crate::toml::PyprojectPrettyFormatter;
use indexmap::IndexMap;
#[cfg(test)]
use std::any::Any;
use std::default::Default;
use std::fs;
use toml_edit::DocumentMut;
use toml_edit::visit_mut::VisitMut;

#[derive(Debug, PartialEq, Eq)]
pub struct Pip {
    pub converter_options: ConverterOptions,
    pub requirements_files: Vec<String>,
    pub dev_requirements_files: Vec<String>,
    pub is_pip_tools: bool,
}

impl Converter for Pip {
    fn build_uv_pyproject(&self) -> String {
        let pyproject_toml_content =
            fs::read_to_string(self.get_project_path().join("pyproject.toml")).unwrap_or_default();
        let pyproject: PyProject = toml::from_str(pyproject_toml_content.as_str()).unwrap();

        let dev_dependencies = dependencies::get(
            &self.get_project_path(),
            self.dev_requirements_files.clone(),
        );

        let dependency_groups = dev_dependencies.map(|dependencies| {
            IndexMap::from([(
                "dev".to_string(),
                dependencies
                    .iter()
                    .map(|dep| DependencyGroupSpecification::String(dep.to_string()))
                    .collect(),
            )])
        });

        let project = Project {
            // "name" is required by uv.
            name: Some(String::new()),
            // "version" is required by uv.
            version: Some("0.0.1".to_string()),
            dependencies: dependencies::get(
                &self.get_project_path(),
                self.requirements_files.clone(),
            ),
            ..Default::default()
        };

        let uv = Uv {
            package: Some(false),
            constraint_dependencies: self.get_constraint_dependencies(),
            ..Default::default()
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
        if self.is_pip_tools {
            return "pip-tools".to_string();
        }
        "pip".to_string()
    }

    fn get_converter_options(&self) -> &ConverterOptions {
        &self.converter_options
    }

    fn respect_locked_versions(&self) -> bool {
        // There are no locked dependencies for pip, so locked versions are only respected for
        // pip-tools.
        self.is_pip_tools && !self.get_converter_options().ignore_locked_versions
    }

    fn get_migrated_files_to_delete(&self) -> Vec<String> {
        let mut files_to_delete: Vec<String> = Vec::new();

        for requirements_file in self
            .requirements_files
            .iter()
            .chain(&self.dev_requirements_files)
        {
            files_to_delete.push(requirements_file.to_string());

            // For pip-tools, also delete `.txt` files generated from `.in` files.
            if self.is_pip_tools {
                files_to_delete.push(requirements_file.replace(".in", ".txt"));
            }
        }

        files_to_delete
    }

    fn get_constraint_dependencies(&self) -> Option<Vec<String>> {
        if !self.is_pip_tools || self.is_dry_run() || !self.respect_locked_versions() {
            return None;
        }

        if let Some(dependencies) = dependencies::get(
            self.get_project_path().as_path(),
            self.requirements_files
                .clone()
                .into_iter()
                .chain(self.dev_requirements_files.clone())
                .map(|f| f.replace(".in", ".txt"))
                .collect(),
        ) {
            if dependencies.is_empty() {
                return None;
            }
            return Some(dependencies);
        }
        None
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}
