use crate::converters;
use crate::converters::Converter;
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

fn project_already_uses_uv(project_path: &Path) -> (bool, String) {
    if project_path.join("uv.lock").exists() {
        return (true, format!("file \"{}\" found", "uv.lock".bold()));
    }

    let pyproject_toml_path = project_path.join("pyproject.toml");
    if !pyproject_toml_path.exists() {
        return (false, String::new());
    }

    let pyproject_toml_content = match fs::read_to_string(&pyproject_toml_path) {
        Ok(content) => content,
        Err(_) => return (false, String::new()),
    };

    let pyproject_toml: PyProject = match toml::from_str(&pyproject_toml_content) {
        Ok(parsed) => parsed,
        Err(_) => return (false, String::new()),
    };

    if pyproject_toml.tool.is_some_and(|tool| tool.uv.is_some()) {
        return (
            true,
            format!(
                "section \"{}\" found in file \"{}\"",
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
        project_path: &Path,
        requirements_files: Vec<String>,
        dev_requirements_files: Vec<String>,
    ) -> Result<Box<dyn Converter>, String> {
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
                    project_path: project_path.to_path_buf(),
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
                    project_path: project_path.to_path_buf(),
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
                    project_path: project_path.to_path_buf(),
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
                    project_path: project_path.to_path_buf(),
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
    project_path: &Path,
    requirements_files: Vec<String>,
    dev_requirements_files: Vec<String>,
    enforced_package_manager: Option<PackageManager>,
) -> Result<Box<dyn Converter>, String> {
    if !project_path.exists() {
        return Err(format!("{} does not exist.", project_path.display()));
    }

    if !project_path.is_dir() {
        return Err(format!("{} is not a directory.", project_path.display()));
    }

    // Always check for uv first
    match project_already_uses_uv(project_path) {
        (true, reason) => return Err(format!("Project is already using uv ({reason})")),
        (false, _) => (),
    }

    if let Some(enforced_package_manager) = enforced_package_manager {
        return match enforced_package_manager.detected(
            project_path,
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
            project_path,
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
    use rstest::rstest;
    use std::path::PathBuf;

    #[rstest]
    #[case("tests/fixtures/poetry/full")]
    #[case("tests/fixtures/poetry/minimal")]
    fn test_auto_detect_poetry_ok(#[case] project_path: &str) {
        let converter = get_converter(
            Path::new(project_path),
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
            &converters::poetry::Poetry {
                project_path: PathBuf::from(project_path)
            }
        );
    }

    #[rstest]
    #[case("tests/fixtures/pipenv/full")]
    #[case("tests/fixtures/pipenv/minimal")]
    fn test_auto_detect_pipenv_ok(#[case] project_path: &str) {
        let converter = get_converter(
            Path::new(project_path),
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
            &converters::pipenv::Pipenv {
                project_path: PathBuf::from(project_path)
            }
        );
    }

    #[test]
    fn test_auto_detect_pip_tools_ok() {
        let converter = get_converter(
            Path::new("tests/fixtures/pip_tools/full"),
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
                project_path: PathBuf::from("tests/fixtures/pip_tools/full"),
                requirements_files: vec!["requirements.in".to_string()],
                dev_requirements_files: vec!["requirements-dev.in".to_string()],
                is_pip_tools: true,
            }
        );
    }

    #[test]
    fn test_auto_detect_pip_ok() {
        let converter = get_converter(
            Path::new("tests/fixtures/pip_tools/full"),
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
                project_path: PathBuf::from("tests/fixtures/pip_tools/full"),
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
        let converter = get_converter(
            Path::new(project_path),
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
        let converter = get_converter(
            Path::new(project_path),
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
            &converters::poetry::Poetry {
                project_path: PathBuf::from(project_path)
            }
        );
    }

    #[rstest]
    #[case("tests/fixtures/poetry", format!("Directory does not contain a {} file.", "pyproject.toml".bold()))]
    #[case("tests/fixtures/pipenv/full", format!("{} does not contain a {} section.", "pyproject.toml".bold(), "[tool.poetry]".bold()))]
    fn test_poetry_err(#[case] project_path: &str, #[case] error: String) {
        let converter = get_converter(
            Path::new(project_path),
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
        let converter = get_converter(
            Path::new(project_path),
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
            &converters::pipenv::Pipenv {
                project_path: PathBuf::from(project_path)
            }
        );
    }

    #[test]
    fn test_pipenv_err() {
        let converter = get_converter(
            Path::new("tests/fixtures/pipenv"),
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
        let converter = get_converter(
            Path::new("tests/fixtures/pip_tools/full"),
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
                project_path: PathBuf::from("tests/fixtures/pip_tools/full"),
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
        let converter = get_converter(
            Path::new("tests/fixtures/poetry/full"),
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
        let converter = get_converter(
            Path::new("tests/fixtures/pip/full"),
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
                project_path: PathBuf::from("tests/fixtures/pip/full"),
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
        let converter = get_converter(
            Path::new("tests/fixtures/poetry/full"),
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
        let converter = get_converter(
            Path::new("tests/fixtures/uv/minimal"),
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            None,
        );
        assert_eq!(
            converter.unwrap_err(),
            format!(
                "Project is already using uv (section \"{}\" found in file \"{}\")",
                "[tool.uv]".bold(),
                "pyproject.toml".bold()
            )
        );
    }

    #[test]
    fn test_auto_detect_already_using_uv_by_config() {
        let converter = get_converter(
            Path::new("tests/fixtures/uv/minimal"),
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            None,
        );
        assert_eq!(
            converter.unwrap_err(),
            format!(
                "Project is already using uv (section \"{}\" found in file \"{}\")",
                "[tool.uv]".bold(),
                "pyproject.toml".bold()
            )
        );
    }

    #[test]
    fn test_auto_detect_already_using_uv_by_lock() {
        let converter = get_converter(
            Path::new("tests/fixtures/uv/with_lock"),
            vec!["requirements.txt".to_string()],
            vec!["requirements-dev.txt".to_string()],
            None,
        );
        assert_eq!(
            converter.unwrap_err(),
            format!(
                "Project is already using uv (file \"{}\" found)",
                "uv.lock".bold()
            )
        );
    }
}
