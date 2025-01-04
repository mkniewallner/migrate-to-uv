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
    Pipenv,
    Poetry,
}

impl PackageManager {
    fn detected(&self, project_path: &Path) -> Result<Box<dyn Converter>, String> {
        match self {
            Self::Poetry => {
                let project_file = "pyproject.toml";
                debug!("Checking if project uses Poetry...");

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

                debug!("{} detected as a package manager.", self);
                Ok(Box::new(converters::poetry::Poetry {
                    project_path: project_path.to_path_buf(),
                }))
            }
            Self::Pipenv => {
                let project_file = "Pipfile";
                debug!("Checking if project uses Pipenv...");

                if !project_path.join(project_file).exists() {
                    return Err(format!(
                        "Directory does not contain a {} file.",
                        project_file.bold()
                    ));
                }

                debug!("{} detected as a package manager.", self);
                Ok(Box::new(converters::pipenv::Pipenv {
                    project_path: project_path.to_path_buf(),
                }))
            }
        }
    }
}

impl Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Auto-detects converter to use based on files (and their content) present in the project, or
/// explicitly select the one associated to the package manager that could be enforced in the CLI.
pub fn get_converter(
    project_path: &Path,
    enforced_package_manager: Option<PackageManager>,
) -> Result<Box<dyn Converter>, String> {
    if !project_path.exists() {
        return Err(format!("{} does not exist.", project_path.display()));
    }

    if !project_path.is_dir() {
        return Err(format!("{} is not a directory.", project_path.display()));
    }

    if let Some(enforced_package_manager) = enforced_package_manager {
        return match enforced_package_manager.detected(project_path) {
            Ok(converter) => return Ok(converter),
            Err(e) => Err(e),
        };
    }

    match PackageManager::Poetry.detected(project_path) {
        Ok(converter) => return Ok(converter),
        Err(err) => debug!("{err}"),
    }

    match PackageManager::Pipenv.detected(project_path) {
        Ok(converter) => return Ok(converter),
        Err(err) => debug!("{err}"),
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
        let converter = get_converter(Path::new(project_path), None).unwrap();
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
        let converter = get_converter(Path::new(project_path), None).unwrap();
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
        let converter = get_converter(Path::new(project_path), None);
        assert_eq!(converter.unwrap_err(), error);
    }

    #[rstest]
    #[case("tests/fixtures/poetry/full")]
    #[case("tests/fixtures/poetry/minimal")]
    fn test_poetry_ok(#[case] project_path: &str) {
        let converter =
            get_converter(Path::new(project_path), Some(PackageManager::Poetry)).unwrap();
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
        let converter = get_converter(Path::new(project_path), Some(PackageManager::Poetry));
        assert_eq!(converter.unwrap_err(), error);
    }

    #[rstest]
    #[case("tests/fixtures/pipenv/full")]
    #[case("tests/fixtures/pipenv/minimal")]
    fn test_pipenv_ok(#[case] project_path: &str) {
        let converter =
            get_converter(Path::new(project_path), Some(PackageManager::Pipenv)).unwrap();
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
            Some(PackageManager::Pipenv),
        );
        assert_eq!(
            converter.unwrap_err(),
            format!("Directory does not contain a {} file.", "Pipfile".bold())
        );
    }
}
