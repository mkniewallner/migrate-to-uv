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
    fn detected(&self, project_path: &Path) -> Result<(), String> {
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
            }
        }

        debug!("{} detected as a package manager.", self);
        Ok(())
    }
}

impl Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub struct Detector<'a> {
    pub project_path: &'a Path,
    pub enforced_package_manager: Option<PackageManager>,
}

/// Auto-detects package manager used based on files (and their content) present in the project, or
/// explicitly search for a specific package manager if enforced in the CLI.
impl Detector<'_> {
    pub fn detect(self) -> Result<PackageManager, String> {
        if !self.project_path.exists() {
            return Err(format!("{} does not exist.", self.project_path.display()));
        }

        if !self.project_path.is_dir() {
            return Err(format!(
                "{} is not a directory.",
                self.project_path.display()
            ));
        }

        if let Some(enforced_package_manager) = self.enforced_package_manager {
            return match enforced_package_manager.detected(self.project_path) {
                Ok(()) => Ok(enforced_package_manager),
                Err(e) => Err(e),
            };
        }

        match PackageManager::Poetry.detected(self.project_path) {
            Ok(()) => return Ok(PackageManager::Poetry),
            Err(err) => debug!("{err}"),
        }

        match PackageManager::Pipenv.detected(self.project_path) {
            Ok(()) => return Ok(PackageManager::Pipenv),
            Err(err) => debug!("{err}"),
        }

        Err(
            "Could not determine which package manager is used from the ones that are supported."
                .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("tests/fixtures/poetry/full", PackageManager::Poetry)]
    #[case("tests/fixtures/poetry/minimal", PackageManager::Poetry)]
    #[case("tests/fixtures/pipenv/full", PackageManager::Pipenv)]
    #[case("tests/fixtures/pipenv/minimal", PackageManager::Pipenv)]
    fn test_auto_detect_ok(#[case] project_path: &str, #[case] expected: PackageManager) {
        let detector = Detector {
            project_path: Path::new(project_path),
            enforced_package_manager: None,
        };
        assert_eq!(detector.detect(), Ok(expected));
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
        let detector = Detector {
            project_path: Path::new(project_path),
            enforced_package_manager: None,
        };
        assert_eq!(detector.detect(), Err(error));
    }

    #[rstest]
    #[case("tests/fixtures/poetry/full")]
    #[case("tests/fixtures/poetry/minimal")]
    fn test_poetry_ok(#[case] project_path: &str) {
        let detector = Detector {
            project_path: Path::new(project_path),
            enforced_package_manager: Some(PackageManager::Poetry),
        };
        assert_eq!(detector.detect(), Ok(PackageManager::Poetry));
    }

    #[rstest]
    #[case("tests/fixtures/poetry", format!("Directory does not contain a {} file.", "pyproject.toml".bold()))]
    #[case("tests/fixtures/pipenv/full", format!("{} does not contain a {} section.", "pyproject.toml".bold(), "[tool.poetry]".bold()))]
    fn test_poetry_err(#[case] project_path: &str, #[case] error: String) {
        let detector = Detector {
            project_path: Path::new(project_path),
            enforced_package_manager: Some(PackageManager::Poetry),
        };
        assert_eq!(detector.detect(), Err(error));
    }

    #[rstest]
    #[case("tests/fixtures/pipenv/full")]
    #[case("tests/fixtures/pipenv/minimal")]
    fn test_pipenv_ok(#[case] project_path: &str) {
        let detector = Detector {
            project_path: Path::new(project_path),
            enforced_package_manager: Some(PackageManager::Pipenv),
        };
        assert_eq!(detector.detect(), Ok(PackageManager::Pipenv));
    }

    #[test]
    fn test_pipenv_err() {
        let detector = Detector {
            project_path: Path::new("tests/fixtures/pipenv"),
            enforced_package_manager: Some(PackageManager::Pipenv),
        };
        assert_eq!(
            detector.detect(),
            Err(format!(
                "Directory does not contain a {} file.",
                "Pipfile".bold()
            ))
        );
    }
}
