use crate::converters::poetry::version::PoetryPep440;
use crate::schema::utils::SingleOrVec;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Deserialize, Serialize)]
pub struct Poetry {
    #[serde(rename = "package-mode")]
    pub package_mode: Option<bool>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
    pub license: Option<String>,
    pub maintainers: Option<Vec<String>>,
    pub readme: Option<SingleOrVec<String>>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub classifiers: Option<Vec<String>>,
    pub source: Option<Vec<Source>>,
    pub dependencies: Option<IndexMap<String, DependencySpecification>>,
    pub extras: Option<IndexMap<String, Vec<String>>>,
    #[serde(rename = "dev-dependencies")]
    pub dev_dependencies: Option<IndexMap<String, DependencySpecification>>,
    pub group: Option<IndexMap<String, DependencyGroup>>,
    pub urls: Option<IndexMap<String, String>>,
    pub scripts: Option<IndexMap<String, Script>>,
    pub plugins: Option<IndexMap<String, IndexMap<String, String>>>,
    pub packages: Option<Vec<Package>>,
    pub include: Option<Vec<Include>>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize)]
pub struct DependencyGroup {
    pub dependencies: IndexMap<String, DependencySpecification>,
}

/// Represents a package source: <https://python-poetry.org/docs/repositories/#package-sources>.
#[derive(Deserialize, Serialize)]
pub struct Source {
    pub name: String,
    pub url: Option<String>,
    pub priority: Option<SourcePriority>,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SourcePriority {
    /// <https://python-poetry.org/docs/repositories/#primary-package-sources>.
    Primary,
    /// <https://python-poetry.org/docs/repositories/#supplemental-package-sources>.
    Supplemental,
    /// <https://python-poetry.org/docs/repositories/#explicit-package-sources>.
    Explicit,
    /// <https://python-poetry.org/docs/1.8/repositories/#default-package-source-deprecated>.
    Default,
    /// <https://python-poetry.org/docs/1.8/repositories/#secondary-package-sources-deprecated>.
    Secondary,
}

/// Represents the different ways a script can be defined in Poetry.
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Script {
    String(String),
    // Although not documented, a script can be set as a map, where `callable` is the script to run.
    // An `extra` field also exists, but it doesn't seem to actually do
    // anything (https://github.com/python-poetry/poetry/issues/6892).
    Map { callable: Option<String> },
}

/// Represents the different ways dependencies can be defined in Poetry.
///
/// See <https://python-poetry.org/docs/dependency-specification/> for details.
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum DependencySpecification {
    /// Simple version constraint: <https://python-poetry.org/docs/basic-usage/#specifying-dependencies>.
    String(String),
    /// Complex version constraint: <https://python-poetry.org/docs/dependency-specification/>.
    Map {
        version: Option<String>,
        extras: Option<Vec<String>>,
        markers: Option<String>,
        python: Option<String>,
        platform: Option<String>,
        source: Option<String>,
        git: Option<String>,
        branch: Option<String>,
        rev: Option<String>,
        tag: Option<String>,
        subdirectory: Option<String>,
        path: Option<String>,
        develop: Option<bool>,
        url: Option<String>,
    },
    /// Multiple constraints dependencies: <https://python-poetry.org/docs/dependency-specification/#multiple-constraints-dependencies>.
    Vec(Vec<Self>),
}

impl DependencySpecification {
    pub fn to_pep_508(&self) -> String {
        match self {
            Self::String(version) => PoetryPep440::from_str(version).unwrap().to_string(),
            Self::Map {
                version, extras, ..
            } => {
                let mut pep_508_version = String::new();

                if let Some(extras) = extras {
                    pep_508_version.push_str(format!("[{}]", extras.join(", ")).as_str());
                }

                if let Some(version) = version {
                    pep_508_version.push_str(
                        PoetryPep440::from_str(version)
                            .unwrap()
                            .to_string()
                            .as_str(),
                    );
                }

                if let Some(marker) = self.get_marker() {
                    pep_508_version.push_str(format!(" ; {marker}").as_str());
                }

                pep_508_version
            }
            Self::Vec(_) => String::new(),
        }
    }

    pub fn get_marker(&self) -> Option<String> {
        let mut combined_markers: Vec<String> = Vec::new();

        if let Self::Map {
            python,
            markers,
            platform,
            ..
        } = self
        {
            if let Some(python) = python {
                combined_markers.push(PoetryPep440::from_str(python).unwrap().to_python_marker());
            }

            if let Some(markers) = markers {
                combined_markers.push(markers.to_string());
            }

            if let Some(platform) = platform {
                combined_markers.push(format!("sys_platform == '{platform}'"));
            }
        }

        if combined_markers.is_empty() {
            return None;
        }
        Some(combined_markers.join(" and "))
    }
}

/// Package distribution definition <https://python-poetry.org/docs/pyproject/#packages>.
#[derive(Deserialize, Serialize)]
pub struct Package {
    pub include: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub format: Option<SingleOrVec<Format>>,
}

/// Package distribution file inclusion: <https://python-poetry.org/docs/pyproject/#exclude-and-include>.
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Include {
    String(String),
    Map {
        path: String,
        format: Option<SingleOrVec<Format>>,
    },
}

#[derive(Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Sdist,
    Wheel,
}

#[derive(Deserialize)]
pub struct PoetryLock {
    pub package: Option<Vec<LockedPackage>>,
}

#[derive(Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
}
