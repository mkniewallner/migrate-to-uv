use crate::schema::poetry::{DependencySpecification, Source, SourcePriority};
use crate::schema::uv::{Index, SourceIndex};

pub fn get_source_index(dependency_specification: &DependencySpecification) -> Option<SourceIndex> {
    match dependency_specification {
        DependencySpecification::Map {
            source: Some(source),
            ..
        } => Some(SourceIndex {
            index: Some(source.to_string()),
            ..Default::default()
        }),
        DependencySpecification::Map { url: Some(url), .. } => Some(SourceIndex {
            url: Some(url.to_string()),
            ..Default::default()
        }),
        DependencySpecification::Map {
            path: Some(path),
            develop,
            ..
        } => Some(SourceIndex {
            path: Some(path.to_string()),
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
    Some(
        poetry_sources?
            .iter()
            .map(|source| Index {
                name: source.name.clone(),
                url: match source.name.to_lowercase().as_str() {
                    "pypi" => Some("https://pypi.org/simple/".to_string()),
                    _ => source.url.clone(),
                },
                default: match source.priority {
                    Some(SourcePriority::Default | SourcePriority::Primary) => Some(true),
                    _ => None,
                },
                explicit: match source.priority {
                    Some(SourcePriority::Explicit) => Some(true),
                    _ => None,
                },
            })
            .collect(),
    )
}
