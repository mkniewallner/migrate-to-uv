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
