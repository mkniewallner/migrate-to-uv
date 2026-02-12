use indexmap::IndexMap;
use regex::Regex;
use std::sync::LazyLock;

static DEPENDENCY_NAME_NORMALIZATION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[-_.]+").unwrap());

pub fn non_empty_vec<T>(vec: Vec<T>) -> Option<Vec<T>> {
    if vec.is_empty() {
        return None;
    }
    Some(vec)
}

pub fn non_empty_index_map<T, U>(map: IndexMap<T, U>) -> Option<IndexMap<T, U>> {
    if map.is_empty() {
        return None;
    }
    Some(map)
}

/// Normalize dependency name following
/// <https://packaging.python.org/en/latest/specifications/name-normalization/#name-normalization>.
pub fn normalize_dependency_name(name: &str) -> String {
    DEPENDENCY_NAME_NORMALIZATION_REGEX
        .replace(name, "-")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("friendly-bard")]
    #[case("Friendly-Bard")]
    #[case("FRIENDLY-BARD")]
    #[case("friendly.bard")]
    #[case("friendly_bard")]
    #[case("friendly--bard")]
    #[case("FrIeNdLy-._.-bArD")]
    fn test_normalize_dependency_name(#[case] name: &str) {
        assert_eq!(normalize_dependency_name(name), "friendly-bard");
    }
}
