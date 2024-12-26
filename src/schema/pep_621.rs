use crate::schema::poetry::Poetry;
use crate::schema::uv::Uv;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// <https://packaging.python.org/en/latest/specifications/pyproject-toml/#pyproject-toml-spec>
#[derive(Default, Deserialize, Serialize)]
pub struct Project {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub authors: Option<Vec<AuthorOrMaintainer>>,
    #[serde(rename(serialize = "requires-python"))]
    pub requires_python: Option<String>,
    pub readme: Option<String>,
    pub license: Option<String>,
    pub maintainers: Option<Vec<AuthorOrMaintainer>>,
    pub keywords: Option<Vec<String>>,
    pub classifiers: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
    #[serde(rename(serialize = "optional-dependencies"))]
    pub optional_dependencies: Option<IndexMap<String, Vec<String>>>,
    pub urls: Option<IndexMap<String, String>>,
    pub scripts: Option<IndexMap<String, String>>,
    #[serde(rename(serialize = "gui-scripts"))]
    pub gui_scripts: Option<IndexMap<String, String>>,
    #[serde(rename(serialize = "entry-points"))]
    pub entry_points: Option<IndexMap<String, IndexMap<String, String>>>,
}

#[derive(Deserialize, Serialize)]
pub struct AuthorOrMaintainer {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Tool {
    pub poetry: Option<Poetry>,
    pub uv: Option<Uv>,
}
