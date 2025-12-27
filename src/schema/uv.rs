use crate::schema::utils::SingleOrVec;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct Uv {
    pub package: Option<bool>,
    /// <https://docs.astral.sh/uv/configuration/indexes/#defining-an-index>
    pub index: Option<Vec<Index>>,
    /// <https://docs.astral.sh/uv/configuration/indexes/#pinning-a-package-to-an-index>
    pub sources: Option<IndexMap<String, SourceContainer>>,
    /// <https://docs.astral.sh/uv/concepts/projects/dependencies/#default-groups>
    #[serde(rename = "default-groups")]
    pub default_groups: Option<Vec<String>>,
    #[serde(rename = "constraint-dependencies")]
    pub constraint_dependencies: Option<Vec<String>>,
    #[serde(rename = "build-backend")]
    pub build_backend: Option<UvBuildBackend>,
}

#[derive(Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct Index {
    pub name: String,
    pub url: Option<String>,
    pub default: Option<bool>,
    pub explicit: Option<bool>,
}

#[derive(Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct SourceIndex {
    pub index: Option<String>,
    pub path: Option<String>,
    pub editable: Option<bool>,
    pub git: Option<String>,
    pub tag: Option<String>,
    pub branch: Option<String>,
    pub rev: Option<String>,
    pub subdirectory: Option<String>,
    pub url: Option<String>,
    pub marker: Option<String>,
}

#[derive(Deserialize, Serialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum SourceContainer {
    SourceIndex(SourceIndex),
    SourceIndexes(Vec<SourceIndex>),
}

#[derive(Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct UvBuildBackend {
    #[serde(rename = "module-name")]
    pub module_name: Option<SingleOrVec<String>>,
    #[serde(rename = "module-root")]
    pub module_root: Option<String>,
    pub namespace: Option<bool>,
    pub data: Option<UvBuildBackendData>,
    #[serde(rename = "default-excludes")]
    pub default_excludes: Option<bool>,
    #[serde(rename = "source-exclude")]
    pub source_exclude: Option<Vec<String>>,
    #[serde(rename = "source-include")]
    pub source_include: Option<Vec<String>>,
    #[serde(rename = "wheel-exclude")]
    pub wheel_exclude: Option<Vec<String>>,
}

#[derive(Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct UvBuildBackendData {
    data: Option<String>,
    headers: Option<String>,
    platlib: Option<String>,
    purelib: Option<String>,
    scripts: Option<String>,
}
