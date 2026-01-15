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

    // If we only have one source that has primary or default priority, we can disable PyPI by
    // setting `default = true`. We filter out sources with explicit priority because they are only
    // used when explicitly requested by packages, and are not looked for otherwise.
    if poetry_sources
        .iter()
        .filter(|s| s.priority != Some(SourcePriority::Explicit))
        .count()
        == 1
        && let Some(SourcePriority::Primary | SourcePriority::Default) | None =
            poetry_sources[0].priority
    {
        let source = &poetry_sources[0];

        // Poetry fails if `pypi` sets a URL, so we can assume that we always have PyPI for this
        // source name. Since PyPI is already enabled by default, there is no need to explicitly
        // have it.
        if source.name.to_lowercase() == "pypi" {
            return None;
        }

        return Some(vec![Index {
            name: source.name.clone(),
            url: source.url.clone(),
            default: Some(true),
            ..Default::default()
        }]);
    }

    Some(
        poetry_sources
            .iter()
            .map(|source| Index {
                name: source.name.clone(),
                url: match source.name.to_lowercase().as_str() {
                    // Poetry fails if `pypi` sets a URL, so we can assume that we always want PyPI
                    // URL for this source name.
                    "pypi" => Some("https://pypi.org/simple/".to_string()),
                    _ => source.url.clone(),
                },
                explicit: match source.priority {
                    Some(SourcePriority::Explicit) => Some(true),
                    _ => None,
                },
                ..Default::default()
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_indexes_single_source_primary() {
        let sources = vec![Source {
            name: "foo".to_string(),
            url: Some("http://foo.bar".to_string()),
            priority: Some(SourcePriority::Primary),
        }];

        let expected = vec![Index {
            name: "foo".to_string(),
            url: Some("http://foo.bar".to_string()),
            default: Some(true),
            ..Default::default()
        }];

        assert_eq!(get_indexes(Some(sources)), Some(expected));
    }

    #[test]
    fn test_get_indexes_single_source_default() {
        let sources = vec![Source {
            name: "foo".to_string(),
            url: Some("http://foo.bar".to_string()),
            priority: Some(SourcePriority::Default),
        }];

        let expected = vec![Index {
            name: "foo".to_string(),
            url: Some("http://foo.bar".to_string()),
            default: Some(true),
            ..Default::default()
        }];

        assert_eq!(get_indexes(Some(sources)), Some(expected));
    }

    #[test]
    fn test_get_indexes_single_source_no_priority() {
        let sources = vec![Source {
            name: "foo".to_string(),
            url: Some("http://foo.bar".to_string()),
            ..Default::default()
        }];

        let expected = vec![Index {
            name: "foo".to_string(),
            url: Some("http://foo.bar".to_string()),
            default: Some(true),
            ..Default::default()
        }];

        assert_eq!(get_indexes(Some(sources)), Some(expected));
    }

    #[test]
    fn test_get_indexes_single_source_primary_pypi() {
        let sources = vec![Source {
            name: "PyPI".to_string(),
            priority: Some(SourcePriority::Primary),
            ..Default::default()
        }];

        assert_eq!(get_indexes(Some(sources)), None);
    }

    #[test]
    fn test_get_indexes_single_source_default_pypi() {
        let sources = vec![Source {
            name: "PyPI".to_string(),
            priority: Some(SourcePriority::Default),
            ..Default::default()
        }];

        assert_eq!(get_indexes(Some(sources)), None);
    }

    #[test]
    fn test_get_indexes_single_source_no_priority_pypi() {
        let sources = vec![Source {
            name: "PyPI".to_string(),
            ..Default::default()
        }];

        assert_eq!(get_indexes(Some(sources)), None);
    }

    #[test]
    fn test_get_indexes_single_source_primary_with_explicit_sources() {
        let sources = vec![
            Source {
                name: "foobar".to_string(),
                url: Some("http://foobar.foo".to_string()),
                priority: Some(SourcePriority::Explicit),
            },
            Source {
                name: "foo".to_string(),
                url: Some("http://foo.bar".to_string()),
                priority: Some(SourcePriority::Primary),
            },
            Source {
                name: "bar".to_string(),
                url: Some("http://bar.foo".to_string()),
                priority: Some(SourcePriority::Explicit),
            },
        ];

        let expected = vec![Index {
            name: "foo".to_string(),
            url: Some("http://foo.bar".to_string()),
            default: Some(true),
            ..Default::default()
        }];

        assert_eq!(get_indexes(Some(sources)), Some(expected));
    }
}
