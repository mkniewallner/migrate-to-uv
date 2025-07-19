use crate::schema::pep_621::Project;
use crate::schema::pep_621::Tool;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct PyProject {
    #[serde(rename = "build-system")]
    pub build_system: Option<BuildSystem>,
    pub project: Option<Project>,
    /// <https://packaging.python.org/en/latest/specifications/dependency-groups/#dependency-groups>
    #[serde(rename = "dependency-groups")]
    pub dependency_groups: Option<IndexMap<String, Vec<DependencyGroupSpecification>>>,
    pub tool: Option<Tool>,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum DependencyGroupSpecification {
    String(String),
    Map {
        #[serde(rename = "include-group")]
        include_group: Option<String>,
    },
}

#[derive(Deserialize, Serialize)]
pub struct BuildSystem {
    pub requires: Vec<String>,
    #[serde(rename = "build-backend")]
    pub build_backend: Option<String>,
}
