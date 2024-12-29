use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Hatch {
    pub build: Option<Build>,
}

#[derive(Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct Build {
    pub targets: Option<IndexMap<String, BuildTarget>>,
}

#[derive(Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct BuildTarget {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub sources: Option<IndexMap<String, String>>,
}
