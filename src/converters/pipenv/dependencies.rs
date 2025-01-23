use crate::converters::{DependencyGroupsAndDefaultGroups, DependencyGroupsStrategy};
use crate::schema;
use crate::schema::pipenv::{DependencySpecification, KeywordMarkers};
use crate::schema::pyproject::DependencyGroupSpecification;
use crate::schema::uv::{SourceContainer, SourceIndex};
use indexmap::IndexMap;

pub fn get(
    pipenv_dependencies: Option<&IndexMap<String, DependencySpecification>>,
    uv_source_index: &mut IndexMap<String, SourceContainer>,
) -> Option<Vec<String>> {
    Some(
        pipenv_dependencies?
            .iter()
            .map(|(name, specification)| {
                let source_index = match specification {
                    DependencySpecification::Map {
                        index: Some(index), ..
                    } => Some(SourceContainer::SourceIndex(SourceIndex {
                        index: Some(index.to_string()),
                        ..Default::default()
                    })),
                    DependencySpecification::Map {
                        path: Some(path),
                        editable,
                        ..
                    } => Some(SourceContainer::SourceIndex(SourceIndex {
                        path: Some(path.to_string()),
                        editable: *editable,
                        ..Default::default()
                    })),
                    DependencySpecification::Map {
                        git: Some(git),
                        ref_,
                        ..
                    } => Some(SourceContainer::SourceIndex(SourceIndex {
                        git: Some(git.clone()),
                        rev: ref_.clone(),
                        ..Default::default()
                    })),
                    _ => None,
                };

                if let Some(source_index) = source_index {
                    uv_source_index.insert(name.to_string(), source_index);
                }

                match specification {
                    DependencySpecification::String(spec) => {
                        format!("{name}{spec}")
                    }
                    DependencySpecification::Map {
                        version,
                        extras,
                        markers,
                        keyword_markers,
                        ..
                    } => {
                        let mut pep_508_version = name.clone();
                        let mut combined_markers: Vec<String> =
                            get_keyword_markers(keyword_markers);

                        if let Some(extras) = extras {
                            pep_508_version.push_str(format!("[{}]", extras.join(", ")).as_str());
                        }

                        if let Some(version) = version {
                            pep_508_version.push_str(version);
                        }

                        if let Some(markers) = markers {
                            combined_markers.push(markers.to_string());
                        }

                        if !combined_markers.is_empty() {
                            pep_508_version.push_str(
                                format!(" ; {}", combined_markers.join(" and ")).as_str(),
                            );
                        }

                        pep_508_version.to_string()
                    }
                }
            })
            .collect(),
    )
}

fn get_keyword_markers(keyword_markers: &KeywordMarkers) -> Vec<String> {
    let mut markers: Vec<String> = Vec::new();

    macro_rules! push_marker {
        ($field:expr, $name:expr) => {
            if let Some(value) = &$field {
                markers.push(format!("{} {}", $name, value));
            }
        };
    }

    push_marker!(keyword_markers.os_name, "os_name");
    push_marker!(keyword_markers.sys_platform, "sys_platform");
    push_marker!(keyword_markers.platform_machine, "platform_machine");
    push_marker!(
        keyword_markers.platform_python_implementation,
        "platform_python_implementation"
    );
    push_marker!(keyword_markers.platform_release, "platform_release");
    push_marker!(keyword_markers.platform_system, "platform_system");
    push_marker!(keyword_markers.platform_version, "platform_version");
    push_marker!(keyword_markers.python_version, "python_version");
    push_marker!(keyword_markers.python_full_version, "python_full_version");
    push_marker!(keyword_markers.implementation_name, "implementation_name");
    push_marker!(
        keyword_markers.implementation_version,
        "implementation_version"
    );

    markers
}

pub fn get_dependency_groups_and_default_groups(
    pipfile: &schema::pipenv::Pipfile,
    uv_source_index: &mut IndexMap<String, SourceContainer>,
    dependency_groups_strategy: DependencyGroupsStrategy,
) -> DependencyGroupsAndDefaultGroups {
    let mut dependency_groups: IndexMap<String, Vec<DependencyGroupSpecification>> =
        IndexMap::new();
    let mut default_groups: Vec<String> = Vec::new();

    // Add dependencies from legacy `[dev-packages]` into `dev` dependency group.
    if let Some(dev_dependencies) = &pipfile.dev_packages {
        dependency_groups.insert(
            "dev".to_string(),
            get(Some(dev_dependencies), uv_source_index)
                .unwrap_or_default()
                .into_iter()
                .map(DependencyGroupSpecification::String)
                .collect(),
        );
    }

    // Add dependencies from `[<category-group>]` into `<category-group>` dependency group,
    // unless `MergeIntoDev` strategy is used, in which case we add them into `dev` dependency
    // group.
    if let Some(category_group) = &pipfile.category_groups {
        for (group, dependency_specification) in category_group {
            dependency_groups
                .entry(match dependency_groups_strategy {
                    DependencyGroupsStrategy::MergeIntoDev => "dev".to_string(),
                    _ => group.to_string(),
                })
                .or_default()
                .extend(
                    get(Some(dependency_specification), uv_source_index)
                        .unwrap_or_default()
                        .into_iter()
                        .map(DependencyGroupSpecification::String),
                );
        }

        match dependency_groups_strategy {
            // When using `SetDefaultGroups` strategy, all dependency groups are referenced in
            // `default-groups` under `[tool.uv]` section. If we only have `dev` dependency group,
            // do not set `default-groups`, as this is already uv's default.
            DependencyGroupsStrategy::SetDefaultGroups => {
                if !dependency_groups.keys().eq(["dev"]) {
                    default_groups.extend(dependency_groups.keys().map(ToString::to_string));
                }
            }
            // When using `IncludeInDev` strategy, dependency groups (except `dev` one) are
            // referenced from `dev` dependency group with `{ include = "<group>" }`.
            DependencyGroupsStrategy::IncludeInDev => {
                dependency_groups
                    .entry("dev".to_string())
                    .or_default()
                    .extend(category_group.keys().filter(|&k| k != "dev").map(|g| {
                        DependencyGroupSpecification::Map {
                            include: Some(g.to_string()),
                        }
                    }));
            }
            _ => (),
        }
    }

    if dependency_groups.is_empty() {
        return (None, None);
    }

    (
        Some(dependency_groups),
        if default_groups.is_empty() {
            None
        } else {
            Some(default_groups)
        },
    )
}
