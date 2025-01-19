use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct Pipfile {
    pub source: Option<Vec<Source>>,
    pub packages: Option<IndexMap<String, DependencySpecification>>,
    #[serde(rename = "dev-packages")]
    pub dev_packages: Option<IndexMap<String, DependencySpecification>>,
    pub requires: Option<Requires>,
    /// Not used, this avoids having the section in `category_groups` below.
    #[allow(dead_code)]
    pipenv: Option<Placeholder>,
    /// Not used, this avoids having the section in `category_groups` below
    #[allow(dead_code)]
    scripts: Option<Placeholder>,
    /// Assume that remaining keys are category groups (<https://pipenv.pypa.io/en/stable/pipfile.html#package-category-groups>).
    #[serde(flatten)]
    pub category_groups: Option<IndexMap<String, IndexMap<String, DependencySpecification>>>,
}

#[derive(Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum DependencySpecification {
    String(String),
    Map {
        version: Option<String>,
        extras: Option<Vec<String>>,
        markers: Option<String>,
        index: Option<String>,
        git: Option<String>,
        #[serde(rename = "ref")]
        ref_: Option<String>,
        path: Option<String>,
        editable: Option<bool>,
        #[serde(flatten)]
        keyword_markers: KeywordMarkers,
    },
}

#[derive(Deserialize)]
pub struct Source {
    pub name: String,
    pub url: String,
}

#[derive(Deserialize)]
pub struct Requires {
    pub python_version: Option<String>,
    pub python_full_version: Option<String>,
}

/// Markers can be set as keywords: <https://github.com/pypa/pipenv/blob/v2024.4.0/pipenv/utils/markers.py#L24-L36>
#[derive(Deserialize)]
pub struct KeywordMarkers {
    pub os_name: Option<String>,
    pub sys_platform: Option<String>,
    pub platform_machine: Option<String>,
    pub platform_python_implementation: Option<String>,
    pub platform_release: Option<String>,
    pub platform_system: Option<String>,
    pub platform_version: Option<String>,
    pub python_version: Option<String>,
    pub python_full_version: Option<String>,
    pub implementation_name: Option<String>,
    pub implementation_version: Option<String>,
}

#[derive(Deserialize)]
pub struct PipenvLock {
    /// Not used, this avoids having the section in `category_groups` below.
    #[allow(dead_code)]
    #[serde(rename = "_meta")]
    meta: Option<Placeholder>,
    #[serde(flatten)]
    pub category_groups: Option<BTreeMap<String, BTreeMap<String, LockedPackage>>>,
}

#[derive(Deserialize)]
pub struct LockedPackage {
    pub version: String,
}

#[derive(Deserialize)]
pub struct Placeholder {}
