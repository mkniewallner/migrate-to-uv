use indexmap::IndexMap;
use std::collections::HashMap;

pub fn get_optional(
    extra_requires: HashMap<String, Option<String>>,
) -> Option<IndexMap<String, Vec<String>>> {
    let mut optional_dependencies: IndexMap<String, Vec<String>> = IndexMap::new();

    for (extra, requires) in extra_requires {
        optional_dependencies.insert(
            extra,
            requires
                .unwrap()
                .trim()
                .lines()
                .map(ToString::to_string)
                .collect(),
        );
    }

    if optional_dependencies.is_empty() {
        return None;
    }
    Some(optional_dependencies)
}
