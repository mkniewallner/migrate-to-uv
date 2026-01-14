use crate::schema::poetry::{DependencySpecification, Source, SourcePriority};
use crate::schema::uv::{Index, SourceIndex};

pub fn get_source_index(dependency_specification: &DependencySpecification) -> Option<SourceIndex> {
    match dependency_specification {
        DependencySpecification::Map {
            source: Some(source),
            ..
        } => Some(SourceIndex {
            index: Some(source.clone()),
            ..Default::default()
        }),
        DependencySpecification::Map { url: Some(url), .. } => Some(SourceIndex {
            url: Some(url.clone()),
            ..Default::default()
        }),
        DependencySpecification::Map {
            path: Some(path),
            develop,
            ..
        } => Some(SourceIndex {
            path: Some(path.clone()),
            editable: *develop,
            ..Default::default()
        }),
        DependencySpecification::Map {
            git: Some(git),
            branch,
            rev,
            tag,
            subdirectory,
            ..
        } => Some(SourceIndex {
            git: Some(git.clone()),
            branch: branch.clone(),
            rev: rev.clone(),
            tag: tag.clone(),
            subdirectory: subdirectory.clone(),
            ..Default::default()
        }),
        _ => None,
    }
}

pub fn get_indexes(poetry_sources: Option<Vec<Source>>) -> Option<Vec<Index>> {
    let mut poetry_sources = poetry_sources?;
    // Sort sources based on the priority, following the order in which Poetry considers
    // sources (https://python-poetry.org/docs/1.8/repositories/#project-configuration). The order
    // is important when converting sources to uv indexes, as uv mostly relies on the indexes
    // position when searching for packages in the different indexes.
    poetry_sources.sort_by(|a, b| {
        // Sources without priority are considered as primary, so if not defined we treat them as
        // primary in the comparison.
        a.priority
            .clone()
            .or(Some(SourcePriority::Primary))
            .cmp(&b.priority.clone().or(Some(SourcePriority::Primary)))
    });

    Some(
        poetry_sources
            .iter()
            .map(|source| Index {
                name: source.name.clone(),
                url: match source.name.to_lowercase().as_str() {
                    "pypi" => Some("https://pypi.org/simple/".to_string()),
                    _ => source.url.clone(),
                },
                default: None,
                explicit: match source.priority {
                    Some(SourcePriority::Explicit) => Some(true),
                    _ => None,
                },
            })
            .collect(),
    )
}
