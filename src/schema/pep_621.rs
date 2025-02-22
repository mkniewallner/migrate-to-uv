use crate::schema::poetry::Poetry;
use crate::schema::uv::Uv;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// <https://packaging.python.org/en/latest/specifications/pyproject-toml/#pyproject-toml-spec>
#[derive(Default, Deserialize, Serialize)]
pub struct Project {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub authors: Option<Vec<AuthorOrMaintainer>>,
    #[serde(rename = "requires-python")]
    pub requires_python: Option<String>,
    pub readme: Option<String>,
    pub license: Option<License>,
    pub maintainers: Option<Vec<AuthorOrMaintainer>>,
    pub keywords: Option<Vec<String>>,
    pub classifiers: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
    #[serde(rename = "optional-dependencies")]
    pub optional_dependencies: Option<IndexMap<String, Vec<String>>>,
    pub urls: Option<IndexMap<String, String>>,
    pub scripts: Option<IndexMap<String, String>>,
    #[serde(rename = "gui-scripts")]
    pub gui_scripts: Option<IndexMap<String, String>>,
    #[serde(rename = "entry-points")]
    pub entry_points: Option<IndexMap<String, IndexMap<String, String>>>,
    #[serde(flatten)]
    pub remaining_fields: HashMap<String, Value>,
}

#[derive(Deserialize, Serialize)]
pub struct AuthorOrMaintainer {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum License {
    String(String),
    Map {
        text: Option<String>,
        file: Option<String>,
    },
}

#[derive(Deserialize, Serialize)]
pub struct Tool {
    pub poetry: Option<Poetry>,
    pub uv: Option<Uv>,
}
