use crate::converters::Converter;
use crate::converters::ConverterOptions;
use crate::converters::pyproject_updater::PyprojectUpdater;
use crate::schema::pep_621::Project;
use crate::schema::pyproject::{DependencyGroupSpecification, PyProject};
use crate::schema::uv::Uv;
use crate::toml::PyprojectPrettyFormatter;
use indexmap::IndexMap;
use log::warn;
use pep508_rs::Requirement;
use std::default::Default;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use toml_edit::DocumentMut;
use toml_edit::visit_mut::VisitMut;
use url::Url;

#[derive(Debug, PartialEq, Eq)]
pub struct Pip {
    pub converter_options: ConverterOptions,
    pub requirements_files: Vec<String>,
    pub dev_requirements_files: Vec<String>,
    pub is_pip_tools: bool,
}

impl Converter for Pip {
    fn build_uv_pyproject(&mut self) -> String {
        let pyproject_toml_content =
            fs::read_to_string(self.get_project_path().join("pyproject.toml")).unwrap_or_default();
        let pyproject: PyProject = toml::from_str(pyproject_toml_content.as_str()).unwrap();

        let dev_dependencies =
            self.get_dependencies(&self.get_project_path(), &self.dev_requirements_files, true);

        let dependency_groups = dev_dependencies.map(|dependencies| {
            IndexMap::from([(
                "dev".to_string(),
                dependencies
                    .iter()
                    .map(|dep| DependencyGroupSpecification::String(dep.to_string()))
                    .collect(),
            )])
        });

        let dependencies =
            self.get_dependencies(&self.get_project_path(), &self.requirements_files, false);
        let project = Project {
            // "name" is required by uv.
            name: Some(String::new()),
            // "version" is required by uv.
            version: Some("0.0.1".to_string()),
            dependencies: if dependencies.is_empty() {
                None
            } else {
                Some(dependencies)
            },
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

        if let Some(dependencies) = self.get_dependencies(
            self.get_project_path().as_path(),
            self.requirements_files
                .clone()
                .into_iter()
                .chain(self.dev_requirements_files.clone())
                .map(|f| f.replace(".in", ".txt"))
                .collect::<Vec<String>>()
                .as_slice(),
            false,
        ) {
            if dependencies.is_empty() {
                return None;
            }
            return Some(dependencies);
        }
        None
    }
}

impl Pip {
    pub fn get_dependencies(
        &mut self,
        project_path: &Path,
        requirements_files: &[String],
        is_dev: bool,
    ) -> Vec<String> {
        let mut dependencies: Vec<String> = Vec::new();

        for requirements_file in requirements_files {
            let requirements_content =
                fs::read_to_string(project_path.join(requirements_file)).unwrap();

            for line in requirements_content.lines() {
                let line = line.trim();

                // Ignore empty lines and comments.
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // https://pip.pypa.io/en/stable/reference/requirements-file-format/#referring-to-other-requirements-files
                // For `-r`, pip allows both `-r requirements.txt` and `-rrequirements.txt`.
                // For `--requirement`, pip only allows `--requirement requirements.txt`.
                // For both options, an infinite number of spaces is allowed between the argument and
                // its value.
                if line.starts_with("-r") || line.starts_with("--requirement ") {
                    let prefix = if line.starts_with("-r") {
                        "-r"
                    } else {
                        "--requirement"
                    };

                    let nested_requirements_file =
                        line.strip_prefix(prefix).unwrap_or_default().trim();

                    // If a referenced requirements file is already passed as an argument, skip it, to
                    // not add dependencies twice.
                    if requirements_files.contains(&nested_requirements_file.to_string()) {
                        continue;
                    }

                    if project_path.join(nested_requirements_file).exists() {
                        dependencies.extend(self.get_dependencies(
                            project_path,
                            &[nested_requirements_file.to_string()],
                            is_dev,
                        ));
                        if is_dev {
                            self.add_dev_requirements_file(nested_requirements_file.to_string());
                        } else {
                            self.add_requirements_file(nested_requirements_file.to_string());
                        }
                    } else {
                        warn!(
                            "Could not resolve \"{nested_requirements_file}\" referenced in \"{requirements_file}\"."
                        );
                    }

                    continue;
                }

                // Ignore lines starting with `-` to ignore other arguments (package names cannot start
                // with a hyphen), as besides `-r`, they are unsupported.
                if line.starts_with('-') {
                    continue;
                }

                let dependency = match line.split_once(" #") {
                    Some((dependency, _)) => dependency,
                    None => line,
                };

                let dependency_specification = Requirement::<Url>::from_str(dependency);

                if let Ok(dependency_specification) = dependency_specification {
                    dependencies.push(dependency_specification.to_string());
                } else {
                    warn!(
                        "Could not parse the following dependency specification as a PEP 508 one: {line}"
                    );
                }
            }
        }

        dependencies
    }

    fn add_requirements_file(&mut self, file: String) {
        self.requirements_files.push(file);
    }

    fn add_dev_requirements_file(&mut self, file: String) {
        self.dev_requirements_files.push(file);
    }
}
