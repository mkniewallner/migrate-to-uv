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

    // Surely there is a better way to handle all the markers in a less
    // repetitive way with serde or a macro, but that will do for now.
    if let Some(os_name) = &keyword_markers.os_name {
        markers.push(format!("os_name {os_name}"));
    }
    if let Some(sys_platform) = &keyword_markers.sys_platform {
        markers.push(format!("sys_platform {sys_platform}"));
    }
    if let Some(platform_machine) = &keyword_markers.platform_machine {
        markers.push(format!("platform_machine {platform_machine}"));
    }
    if let Some(platform_python_implementation) = &keyword_markers.platform_python_implementation {
        markers.push(format!(
            "platform_python_implementation {platform_python_implementation}"
        ));
    }
    if let Some(platform_release) = &keyword_markers.platform_release {
        markers.push(format!("platform_release {platform_release}"));
    }
    if let Some(platform_system) = &keyword_markers.platform_system {
        markers.push(format!("platform_system {platform_system}"));
    }
    if let Some(platform_version) = &keyword_markers.platform_version {
        markers.push(format!("platform_version {platform_version}"));
    }
    if let Some(python_version) = &keyword_markers.python_version {
        markers.push(format!("python_version {python_version}"));
    }
    if let Some(python_full_version) = &keyword_markers.python_full_version {
        markers.push(format!("python_full_version {python_full_version}"));
    }
    if let Some(implementation_name) = &keyword_markers.implementation_name {
        markers.push(format!("implementation_name {implementation_name}"));
    }
    if let Some(implementation_version) = &keyword_markers.implementation_version {
        markers.push(format!("implementation_version {implementation_version}"));
    }

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
