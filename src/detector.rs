use crate::converters;
use crate::converters::{Converter, ConverterOptions};
use crate::schema::pyproject::PyProject;
use log::debug;
use owo_colors::OwoColorize;
use std::fmt::Display;
use std::fs;
use std::path::Path;

/// Lists the package managers supported for the migration.
#[derive(clap::ValueEnum, Clone, Debug, Eq, PartialEq)]
pub enum PackageManager {
    Pip,
    PipTools,
    Pipenv,
    Poetry,
}

/// Check if the projects already uses uv, and return the reason for it if we find that it does.
fn project_already_uses_uv(project_path: &Path) -> (bool, String) {
    // Check for uv.lock first
    if project_path.join("uv.lock").exists() {
        return (true, format!("\"{}\" detected", "uv.lock".bold()));
    }

    // Check for pyproject.toml
    let pyproject_toml_path = project_path.join("pyproject.toml");
    if !pyproject_toml_path.exists() {
        return (false, String::new());
    }

    // Read and parse pyproject.toml, returning false if there are any errors
    let Ok(pyproject_toml_content) = fs::read_to_string(&pyproject_toml_path) else {
        return (false, String::new());
    };

    let Ok(pyproject_toml) = toml::from_str::<PyProject>(&pyproject_toml_content) else {
        return (false, String::new());
    };

    // Check for [tool.uv] section
    if pyproject_toml.tool.is_some_and(|tool| tool.uv.is_some()) {
        return (
            true,
            format!(
                "'{}' section detected in '{}'",
                "[tool.uv]".bold(),
                "pyproject.toml".bold()
            ),
        );
    }

    (false, String::new())
}

impl PackageManager {
    fn detected(
        &self,
        converter_options: &ConverterOptions,
        requirements_files: Vec<String>,
        dev_requirements_files: Vec<String>,
    ) -> Result<Box<dyn Converter>, String> {
        let project_path = &converter_options.project_path;

        debug!("Checking if project uses {self}...");

        match self {
            Self::Poetry => {
                let project_file = "pyproject.toml";

                let pyproject_toml_path = project_path.join(project_file);

                if !pyproject_toml_path.exists() {
                    return Err(format!(
                        "Directory does not contain a {} file.",
                        project_file.bold()
                    ));
                }

                let pyproject_toml_content = fs::read_to_string(pyproject_toml_path).unwrap();
                let pyproject_toml: PyProject =
                    toml::from_str(pyproject_toml_content.as_str()).unwrap();

                if pyproject_toml.tool.is_none_or(|tool| tool.poetry.is_none()) {
                    return Err(format!(
                        "{} does not contain a {} section.",
                        project_file.bold(),
                        "[tool.poetry]".bold()
                    ));
                }

                debug!("{self} detected as a package manager.");
                Ok(Box::new(converters::poetry::Poetry {
                    converter_options: converter_options.clone(),
                }))
            }
            Self::Pipenv => {
                let project_file = "Pipfile";

                if !project_path.join(project_file).exists() {
                    return Err(format!(
                        "Directory does not contain a {} file.",
                        project_file.bold()
                    ));
                }

                debug!("{self} detected as a package manager.");
                Ok(Box::new(converters::pipenv::Pipenv {
                    converter_options: converter_options.clone(),
                }))
            }
            Self::PipTools => {
                let mut found_requirements_files: Vec<String> = Vec::new();
                let mut found_dev_requirements_files: Vec<String> = Vec::new();

                for file in requirements_files {
                    if project_path.join(&file).with_extension("in").exists() {
                        found_requirements_files.push(file.replace(".txt", ".in"));
                    }
                }

                for file in dev_requirements_files {
                    if project_path.join(&file).with_extension("in").exists() {
                        found_dev_requirements_files.push(file.replace(".txt", ".in"));
                    }
                }

                if found_requirements_files.is_empty() && found_dev_requirements_files.is_empty() {
                    return Err(
                        "Directory does not contain any pip-tools requirements file.".to_string(),
                    );
                }

                debug!("{self} detected as a package manager.");
                Ok(Box::new(converters::pip::Pip {
                    converter_options: converter_options.clone(),
                    requirements_files: found_requirements_files,
                    dev_requirements_files: found_dev_requirements_files,
                    is_pip_tools: true,
                }))
            }
            Self::Pip => {
                let mut found_requirements_files: Vec<String> = Vec::new();
                let mut found_dev_requirements_files: Vec<String> = Vec::new();

                for file in requirements_files {
                    if project_path.join(&file).exists() {
                        found_requirements_files.push(file);
                    }
                }

                for file in dev_requirements_files {
                    if project_path.join(&file).exists() {
                        found_dev_requirements_files.push(file);
                    }
                }

                if found_requirements_files.is_empty() && found_dev_requirements_files.is_empty() {
                    return Err("Directory does not contain any pip requirements file.".to_string());
                }

                debug!("{self} detected as a package manager.");
                Ok(Box::new(converters::pip::Pip {
                    converter_options: converter_options.clone(),
                    requirements_files: found_requirements_files,
                    dev_requirements_files: found_dev_requirements_files,
                    is_pip_tools: false,
                }))
            }
        }
    }
}

impl Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pip => write!(f, "pip"),
            Self::PipTools => write!(f, "pip-tools"),
            Self::Pipenv => write!(f, "Pipenv"),
            Self::Poetry => write!(f, "Poetry"),
        }
    }
}

/// Auto-detects converter to use based on files (and their content) present in the project, or
/// explicitly select the one associated to the package manager that could be enforced in the CLI.
pub fn get_converter(
    converter_options: &ConverterOptions,
    requirements_files: Vec<String>,
    dev_requirements_files: Vec<String>,
    enforced_package_manager: Option<PackageManager>,
) -> Result<Box<dyn Converter>, String> {
    let project_path = &converter_options.project_path;

    if !project_path.exists() {
        return Err(format!("{} does not exist.", project_path.display()));
    }

    if !project_path.is_dir() {
        return Err(format!("{} is not a directory.", project_path.display()));
    }

    if !converter_options.skip_uv_checks {
        // Check if `uv` exists,
        // except for when the user explicitly asks
        // to skip these checks.
        if let (true, reason) = project_already_uses_uv(project_path.as_path()) {
            return Err(format!("Project is already using uv ({reason})"));
        }
    }

    if let Some(enforced_package_manager) = enforced_package_manager {
        return match enforced_package_manager.detected(
            converter_options,
            requirements_files,
            dev_requirements_files,
        ) {
            Ok(converter) => return Ok(converter),
            Err(e) => Err(e),
        };
    }

    for package_manager in [
        PackageManager::Poetry,
        PackageManager::Pipenv,
        PackageManager::PipTools,
        PackageManager::Pip,
    ] {
        match package_manager.detected(
            converter_options,
            requirements_files.clone(),
            dev_requirements_files.clone(),
        ) {
            Ok(converter) => return Ok(converter),
            Err(err) => debug!("{err}"),
        }
    }

    Err(
        "Could not determine which package manager is used from the ones that are supported."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converters::DependencyGroupsStrategy;
    use rstest::rstest;
    use std::path::PathBuf;

    const fn get_converter_options(project_path: PathBuf) -> ConverterOptions {
        ConverterOptions {
            project_path,
            dry_run: true,
            skip_lock: true,
            skip_uv_checks: false,
            ignore_locked_versions: false,
            replace_project_section: false,
            keep_old_metadata: false,
            dependency_groups_strategy: DependencyGroupsStrategy::SetDefaultGroups,
        }
    }

    #[rstest]
    #[case("tests/fixtures/poetry/full")]
    #[case("tests/fixtures/poetry/minimal")]
    fn test_auto_detect_poetry_ok(#[case] project_path: &str) {
        let converter_options = get_converter_options(PathBuf::from(project_path));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            None,
        )
        .unwrap();
        assert_eq!(
            converter
                .as_any()
                .downcast_ref::<converters::poetry::Poetry>()
                .unwrap(),
            &converters::poetry::Poetry { converter_options }
        );
    }

    #[rstest]
    #[case("tests/fixtures/pipenv/full")]
    #[case("tests/fixtures/pipenv/minimal")]
    fn test_auto_detect_pipenv_ok(#[case] project_path: &str) {
        let converter_options = get_converter_options(PathBuf::from(project_path));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            None,
        )
        .unwrap();
        assert_eq!(
            converter
                .as_any()
                .downcast_ref::<converters::pipenv::Pipenv>()
                .unwrap(),
            &converters::pipenv::Pipenv { converter_options }
        );
    }

    #[test]
    fn test_auto_detect_pip_tools_ok() {
        let converter_options =
            get_converter_options(PathBuf::from("tests/fixtures/pip_tools/full"));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            None,
        )
        .unwrap();
        assert_eq!(
            converter
                .as_any()
                .downcast_ref::<converters::pip::Pip>()
                .unwrap(),
            &converters::pip::Pip {
                converter_options,
                requirements_files: vec!["requirements.in".to_string()],
                dev_requirements_files: vec!["requirements-dev.in".to_string()],
                is_pip_tools: true,
            }
        );
    }

    #[test]
    fn test_auto_detect_pip_ok() {
        let converter_options =
            get_converter_options(PathBuf::from("tests/fixtures/pip_tools/full"));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            None,
        )
        .unwrap();
        assert_eq!(
            converter
                .as_any()
                .downcast_ref::<converters::pip::Pip>()
                .unwrap(),
            &converters::pip::Pip {
                converter_options,
                requirements_files: vec!["requirements.in".to_string()],
                dev_requirements_files: vec!["requirements-dev.in".to_string()],
                is_pip_tools: true,
            }
        );
    }

    #[rstest]
    #[case(
        "tests/fixtures/non_existing_path",
        "tests/fixtures/non_existing_path does not exist."
    )]
    #[case(
        "tests/fixtures/poetry/full/pyproject.toml",
        "tests/fixtures/poetry/full/pyproject.toml is not a directory."
    )]
    #[case(
        "tests/fixtures/poetry",
        "Could not determine which package manager is used from the ones that are supported."
    )]
    fn test_auto_detect_err(#[case] project_path: &str, #[case] error: String) {
        let converter_options = get_converter_options(PathBuf::from(project_path));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            None,
        );
        assert_eq!(converter.unwrap_err(), error);
    }

    #[rstest]
    #[case("tests/fixtures/poetry/full")]
    #[case("tests/fixtures/poetry/minimal")]
    fn test_poetry_ok(#[case] project_path: &str) {
        let converter_options = get_converter_options(PathBuf::from(project_path));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            Some(PackageManager::Poetry),
        )
        .unwrap();
        assert_eq!(
            converter
                .as_any()
                .downcast_ref::<converters::poetry::Poetry>()
                .unwrap(),
            &converters::poetry::Poetry { converter_options }
        );
    }

    #[rstest]
    #[case("tests/fixtures/poetry", format!("Directory does not contain a {} file.", "pyproject.toml".bold()))]
    #[case("tests/fixtures/pipenv/full", format!("{} does not contain a {} section.", "pyproject.toml".bold(), "[tool.poetry]".bold()))]
    fn test_poetry_err(#[case] project_path: &str, #[case] error: String) {
        let converter_options = get_converter_options(PathBuf::from(project_path));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            Some(PackageManager::Poetry),
        );
        assert_eq!(converter.unwrap_err(), error);
    }

    #[rstest]
    #[case("tests/fixtures/pipenv/full")]
    #[case("tests/fixtures/pipenv/minimal")]
    fn test_pipenv_ok(#[case] project_path: &str) {
        let converter_options = get_converter_options(PathBuf::from(project_path));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            Some(PackageManager::Pipenv),
        )
        .unwrap();
        assert_eq!(
            converter
                .as_any()
                .downcast_ref::<converters::pipenv::Pipenv>()
                .unwrap(),
            &converters::pipenv::Pipenv { converter_options }
        );
    }

    #[test]
    fn test_pipenv_err() {
        let converter_options = get_converter_options(PathBuf::from("tests/fixtures/pipenv"));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            Some(PackageManager::Pipenv),
        );
        assert_eq!(
            converter.unwrap_err(),
            format!("Directory does not contain a {} file.", "Pipfile".bold())
        );
    }

    #[test]
    fn test_pip_tools_ok() {
        let converter_options =
            get_converter_options(PathBuf::from("tests/fixtures/pip_tools/full"));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.in".to_string()],
            vec![
                "requirements-dev.in".to_string(),
                "requirements-typing.in".to_string(),
            ],
            Some(PackageManager::PipTools),
        )
        .unwrap();
        assert_eq!(
            converter
                .as_any()
                .downcast_ref::<converters::pip::Pip>()
                .unwrap(),
            &converters::pip::Pip {
                converter_options,
                requirements_files: vec!["requirements.in".to_string()],
                dev_requirements_files: vec![
                    "requirements-dev.in".to_string(),
                    "requirements-typing.in".to_string()
                ],
                is_pip_tools: true,
            }
        );
    }

    #[test]
    fn test_pip_tools_err() {
        let converter_options = get_converter_options(PathBuf::from("tests/fixtures/poetry/full"));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.in".to_string()],
            vec![
                "requirements-dev.in".to_string(),
                "requirements-typing.in".to_string(),
            ],
            Some(PackageManager::PipTools),
        );
        assert_eq!(
            converter.unwrap_err(),
            "Directory does not contain any pip-tools requirements file.",
        );
    }

    #[test]
    fn test_pip_ok() {
        let converter_options = get_converter_options(PathBuf::from("tests/fixtures/pip/full"));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec![
                "requirements-dev.txt".to_string(),
                "requirements-typing.txt".to_string(),
            ],
            Some(PackageManager::Pip),
        )
        .unwrap();
        assert_eq!(
            converter
                .as_any()
                .downcast_ref::<converters::pip::Pip>()
                .unwrap(),
            &converters::pip::Pip {
                converter_options,
                requirements_files: vec!["requirements.txt".to_string()],
                dev_requirements_files: vec![
                    "requirements-dev.txt".to_string(),
                    "requirements-typing.txt".to_string()
                ],
                is_pip_tools: false,
            }
        );
    }

    #[test]
    fn test_pip_err() {
        let converter_options = get_converter_options(PathBuf::from("tests/fixtures/poetry/full"));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec![
                "requirements-dev.txt".to_string(),
                "requirements-typing.txt".to_string(),
            ],
            Some(PackageManager::Pip),
        );
        assert_eq!(
            converter.unwrap_err(),
            "Directory does not contain any pip requirements file.",
        );
    }

    #[test]
    fn test_auto_detect_already_using_uv() {
        let converter_options = get_converter_options(PathBuf::from("tests/fixtures/uv/minimal"));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            None,
        );
        assert_eq!(
            converter.unwrap_err(),
            format!(
                "Project is already using uv ('{}' section detected in '{}')",
                "[tool.uv]".bold(),
                "pyproject.toml".bold()
            )
        );
    }

    #[test]
    fn test_skip_auto_detect_using_uv() {
        let mut converter_options =
            get_converter_options(PathBuf::from("tests/fixtures/uv/minimal"));
        converter_options.skip_uv_checks = true;

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            None,
        );
        assert_eq!(
            converter.unwrap_err(),
            "Could not determine which package manager is used from the ones that are supported."
                .to_string(),
        );
    }

    #[test]
    fn test_auto_detect_already_using_uv_by_config() {
        let converter_options = get_converter_options(PathBuf::from("tests/fixtures/uv/minimal"));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            None,
        );
        assert_eq!(
            converter.unwrap_err(),
            format!(
                "Project is already using uv ('{}' section detected in '{}')",
                "[tool.uv]".bold(),
                "pyproject.toml".bold()
            )
        );
    }

    #[test]
    fn test_auto_detect_already_using_uv_by_lock() {
        let converter_options = get_converter_options(PathBuf::from("tests/fixtures/uv/with_lock"));

        let converter = get_converter(
            &converter_options,
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            None,
        );
        assert_eq!(
            converter.unwrap_err(),
            format!(
                "Project is already using uv (\"{}\" detected)",
                "uv.lock".bold()
            )
        );
    }
}
