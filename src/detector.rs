use crate::converters;
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
    PipTools,
    Poetry,
}

impl PackageManager {
    fn detected(
        &self,
        project_path: &Path,
        requirement_files: Vec<String>,
        dev_requirement_files: Vec<String>,
    ) -> Result<Box<dyn converters::Converter>, String> {
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

                for file in requirement_files {
                    if project_path.join(&file).exists() {
                        found_requirements_files.push(file);
                    }
                }

                for file in dev_requirement_files {
                    if project_path.join(&file).exists() {
                        found_dev_requirements_files.push(file);
                    }
                }

                if found_requirements_files.is_empty() && found_dev_requirements_files.is_empty() {
                    return Err(
                        "Directory does not contain any pip-tools requirements file.".to_string(),
                    );
                }

                debug!("{self} detected as a package manager.");
                Ok(Box::new(converters::pip_tools::PipTools {
                    project_path: project_path.to_path_buf(),
                    requirements_files: found_requirements_files,
                    dev_requirements_files: found_dev_requirements_files,
                }))
            }
        }
    }
}

impl Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pipenv => write!(f, "Pipenv"),
            Self::PipTools => write!(f, "pip-tools"),
            Self::Poetry => write!(f, "Poetry"),
        }
    }
}

pub struct Detector<'a> {
    pub project_path: &'a Path,
    pub requirement_files: Vec<String>,
    pub dev_requirement_files: Vec<String>,
    pub enforced_package_manager: Option<PackageManager>,
}

/// Auto-detects package manager used based on files (and their content) present in the project, or
/// explicitly search for a specific package manager if enforced in the CLI.
impl Detector<'_> {
    pub fn detect(self) -> Result<Box<dyn converters::Converter>, String> {
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
            return match enforced_package_manager.detected(
                self.project_path,
                self.requirement_files,
                self.dev_requirement_files,
            ) {
                Ok(converter) => Ok(converter),
                Err(e) => Err(e),
            };
        }

        match PackageManager::Poetry.detected(
            self.project_path,
            self.requirement_files.clone(),
            self.dev_requirement_files.clone(),
        ) {
            Ok(converter) => return Ok(converter),
            Err(err) => debug!("{err}"),
        }

        match PackageManager::Pipenv.detected(
            self.project_path,
            self.requirement_files.clone(),
            self.dev_requirement_files.clone(),
        ) {
            Ok(converter) => return Ok(converter),
            Err(err) => debug!("{err}"),
        }

        match PackageManager::PipTools.detected(
            self.project_path,
            self.requirement_files.clone(),
            self.dev_requirement_files.clone(),
        ) {
            Ok(converter) => return Ok(converter),
            Err(err) => debug!("{err}"),
        }

        Err(
            "Could not determine which package manager is used from the ones that are supported."
                .to_string(),
        )
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use rstest::rstest;
//     use std::path::PathBuf;
//
//     #[rstest]
//     #[case("tests/fixtures/poetry/full", Box::new(converters::poetry::Poetry{project_path: PathBuf::from("tests/fixtures/poetry/full/pyproject.toml")}))]
//     #[case("tests/fixtures/poetry/minimal", Box::new(converters::poetry::Poetry{project_path: PathBuf::from("tests/fixtures/poetry/minimal/pyproject.toml")}))]
//     #[case("tests/fixtures/pipenv/full", Box::new(converters::pipenv::Pipenv{project_path: PathBuf::from("tests/fixtures/pipenv/full/pyproject.toml")}))]
//     #[case("tests/fixtures/pipenv/minimal", Box::new(converters::pipenv::Pipenv{project_path: PathBuf::from("tests/fixtures/pipenv/minimal/pyproject.toml")}))]
//     fn test_auto_detect_ok(
//         #[case] project_path: &str,
//         #[case] expected: Box<dyn converters::Converter>,
//     ) {
//         let detector = Detector {
//             project_path: Path::new(project_path),
//             requirement_files: vec!["requirements.in".to_string()],
//             dev_requirement_files: vec!["requirements-dev.in".to_string()],
//             enforced_package_manager: None,
//         };
//         assert_eq!(detector.detect(), Ok(expected));
//     }
//
//     #[rstest]
//     #[case(
//         "tests/fixtures/non_existing_path",
//         "tests/fixtures/non_existing_path does not exist."
//     )]
//     #[case(
//         "tests/fixtures/poetry/full/pyproject.toml",
//         "tests/fixtures/poetry/full/pyproject.toml is not a directory."
//     )]
//     #[case(
//         "tests/fixtures/poetry",
//         "Could not determine which package manager is used from the ones that are supported."
//     )]
//     fn test_auto_detect_err(#[case] project_path: &str, #[case] error: String) {
//         let detector = Detector {
//             project_path: Path::new(project_path),
//             requirement_files: vec!["requirements.in".to_string()],
//             dev_requirement_files: vec!["requirements-dev.in".to_string()],
//             enforced_package_manager: None,
//         };
//         assert_eq!(detector.detect(), Err(error));
//     }
//
//     #[rstest]
//     #[case("tests/fixtures/poetry/full", Box::new(converters::poetry::Poetry{project_path: PathBuf::from("tests/fixtures/poetry/full/pyproject.toml")}))]
//     #[case("tests/fixtures/poetry/minimal", Box::new(converters::poetry::Poetry{project_path: PathBuf::from("tests/fixtures/poetry/minimal/pyproject.toml")}))]
//     fn test_poetry_ok(
//         #[case] project_path: &str,
//         #[case] expected: Box<dyn converters::Converter>,
//     ) {
//         let detector = Detector {
//             project_path: Path::new(project_path),
//             requirement_files: vec!["requirements.in".to_string()],
//             dev_requirement_files: vec!["requirements-dev.in".to_string()],
//             enforced_package_manager: Some(PackageManager::Poetry),
//         };
//         assert_eq!(detector.detect(), Ok(expected));
//     }
//
//     #[rstest]
//     #[case("tests/fixtures/poetry", format!("Directory does not contain a {} file.", "pyproject.toml".bold()))]
//     #[case("tests/fixtures/pipenv/full", format!("{} does not contain a {} section.", "pyproject.toml".bold(), "[tool.poetry]".bold()))]
//     fn test_poetry_err(#[case] project_path: &str, #[case] error: String) {
//         let detector = Detector {
//             project_path: Path::new(project_path),
//             requirement_files: vec!["requirements.in".to_string()],
//             dev_requirement_files: vec!["requirements-dev.in".to_string()],
//             enforced_package_manager: Some(PackageManager::Poetry),
//         };
//         assert_eq!(detector.detect(), Err(error));
//     }
//
//     #[rstest]
//     #[case("tests/fixtures/pipenv/full", Box::new(converters::poetry::Poetry{project_path: PathBuf::from("tests/fixtures/pipenv/full/pyproject.toml")}))]
//     #[case("tests/fixtures/pipenv/minimal", Box::new(converters::poetry::Poetry{project_path: PathBuf::from("tests/fixtures/pipenv/full/pyproject.toml")}))]
//     fn test_pipenv_ok(
//         #[case] project_path: &str,
//         #[case] expected: Box<dyn converters::Converter>,
//     ) {
//         let detector = Detector {
//             project_path: Path::new(project_path),
//             requirement_files: vec!["requirements.in".to_string()],
//             dev_requirement_files: vec!["requirements-dev.in".to_string()],
//             enforced_package_manager: Some(PackageManager::Pipenv),
//         };
//         assert_eq!(detector.detect(), Ok(expected));
//     }
//
//     #[test]
//     fn test_pipenv_err() {
//         let detector = Detector {
//             project_path: Path::new("tests/fixtures/pipenv"),
//             requirement_files: vec!["requirements.in".to_string()],
//             dev_requirement_files: vec!["requirements-dev.in".to_string()],
//             enforced_package_manager: Some(PackageManager::Pipenv),
//         };
//         assert_eq!(
//             detector.detect(),
//             Err(format!(
//                 "Directory does not contain a {} file.",
//                 "Pipfile".bold()
//             ))
//         );
//     }
// }
