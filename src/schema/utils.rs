use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum SingleOrVec<T> {
    Single(T),
    Vec(Vec<T>),
}
