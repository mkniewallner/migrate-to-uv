use crate::converters::poetry::sources;
use crate::converters::{DependencyGroupsAndDefaultGroups, DependencyGroupsStrategy};
use crate::errors::{add_recoverable_error, add_unrecoverable_error};
use crate::schema;
use crate::schema::poetry::DependencySpecification;
use crate::schema::pyproject::DependencyGroupSpecification;
use crate::schema::uv::{SourceContainer, SourceIndex};
use indexmap::IndexMap;
use owo_colors::OwoColorize;
use std::collections::HashSet;

pub fn get(
    poetry_dependencies: Option<&IndexMap<String, DependencySpecification>>,
    uv_source_index: &mut IndexMap<String, SourceContainer>,
) -> Option<Vec<String>> {
    let poetry_dependencies = poetry_dependencies?;
    let mut dependencies: Vec<String> = Vec::new();

    for (name, specification) in poetry_dependencies {
        match specification {
            DependencySpecification::String(_) => match specification.to_pep_508() {
                Ok(v) => dependencies.push(format!("{name}{v}")),
                Err(e) => add_unrecoverable_error(e.format(name)),
            },
            DependencySpecification::Map { .. } => {
                let source_index = sources::get_source_index(specification);

                if let Some(source_index) = source_index {
                    uv_source_index
                        .insert(name.clone(), SourceContainer::SourceIndex(source_index));
                }

                match specification.to_pep_508() {
                    Ok(v) => dependencies.push(format!("{name}{v}")),
                    Err(e) => add_unrecoverable_error(e.format(name)),
                }
            }
            // Multiple constraints dependencies: https://python-poetry.org/docs/dependency-specification#multiple-constraints-dependencies
            DependencySpecification::Vec(specs) => {
                let mut source_indexes: Vec<SourceIndex> = Vec::new();

                for spec in specs {
                    let source_index = sources::get_source_index(spec);

                    // When using multiple constraints and a source is set, markers apply to the
                    // source, not the dependency. So if we find both a source and a marker, we
                    // apply the marker to the source.
                    if let Some(mut source_index) = source_index {
                        if let DependencySpecification::Map {
                            python,
                            platform,
                            markers,
                            ..
                        } = spec
                            && (python.is_some() || platform.is_some() || markers.is_some())
                        {
                            source_index.marker = spec.get_marker();
                        }

                        source_indexes.push(source_index);
                    }
                }

                // If no source was found on any of the dependency specification, we add the
                // different variants of the dependencies with their respective markers. Otherwise,
                // we add the different variants of the sources with their respective markers.
                if source_indexes.is_empty() {
                    for spec in specs {
                        match spec.to_pep_508() {
                            Ok(v) => dependencies.push(format!("{name}{v}")),
                            Err(e) => add_unrecoverable_error(e.format(name)),
                        }
                    }
                } else {
                    uv_source_index
                        .insert(name.clone(), SourceContainer::SourceIndexes(source_indexes));

                    dependencies.push(name.clone());
                }
            }
        }
    }

    if dependencies.is_empty() {
        return None;
    }

    Some(dependencies)
}

pub fn get_optional(
    poetry_dependencies: &mut Option<IndexMap<String, DependencySpecification>>,
    extras: Option<IndexMap<String, Vec<String>>>,
) -> Option<IndexMap<String, Vec<String>>> {
    let extras = extras?;
    let poetry_dependencies = poetry_dependencies.as_mut()?;

    let mut dependencies_to_remove: HashSet<&str> = HashSet::new();

    let optional_dependencies: IndexMap<String, Vec<String>> = extras
        .iter()
        .map(|(extra, extra_dependencies)| {
            (
                extra.clone(),
                extra_dependencies
                    .iter()
                    .filter_map(|dependency| {
                        // If dependency listed in extra does not exist, warn the user.
                        poetry_dependencies.get(dependency).map_or_else(
                            || {
                                add_recoverable_error(format!(
                                    "Could not find dependency \"{}\" listed in \"{}\" extra.",
                                    dependency.bold(),
                                    extra.bold()
                                ));
                                None
                            },
                            |dependency_specification| {
                                dependencies_to_remove.insert(dependency);
                                Some(format!(
                                    "{}{}",
                                    dependency,
                                    dependency_specification.to_pep_508().unwrap(),
                                ))
                            },
                        )
                    })
                    .collect(),
            )
        })
        .collect();

    if optional_dependencies.is_empty() {
        return None;
    }

    for dep in dependencies_to_remove {
        let _ = &mut poetry_dependencies.shift_remove(dep);
    }

    Some(optional_dependencies)
}

pub fn get_dependency_groups_and_default_groups(
    poetry: &schema::poetry::Poetry,
    uv_source_index: &mut IndexMap<String, SourceContainer>,
    dependency_groups_strategy: DependencyGroupsStrategy,
) -> DependencyGroupsAndDefaultGroups {
    let mut dependency_groups: IndexMap<String, Vec<DependencyGroupSpecification>> =
        IndexMap::new();
    let mut default_groups: Vec<String> = Vec::new();

    // Add dependencies from legacy `[poetry.dev-dependencies]` into `dev` dependency group.
    if let Some(dev_dependencies) = &poetry.dev_dependencies {
        dependency_groups.insert(
            "dev".to_string(),
            get(Some(dev_dependencies), uv_source_index)
                .unwrap_or_default()
                .into_iter()
                .map(DependencyGroupSpecification::String)
                .collect(),
        );
    }

    // Add dependencies from `[poetry.group.<group>.dependencies]` into `<group>` dependency group,
    // unless `MergeIntoDev` strategy is used, in which case:
    // - we add non-optional groups into `dev` dependency group
    // - we keep the original group for optional groups
    if let Some(poetry_group) = &poetry.group {
        let mut optional_groups = HashSet::new();

        for (group, dependency_group) in poetry_group {
            if dependency_group.optional == Some(true) {
                optional_groups.insert(group.clone());
            }

            dependency_groups
                .entry(match dependency_groups_strategy {
                    DependencyGroupsStrategy::MergeIntoDev if !optional_groups.contains(group) => {
                        "dev".to_string()
                    }
                    _ => group.clone(),
                })
                .or_default()
                .extend(
                    get(Some(&dependency_group.dependencies), uv_source_index)
                        .unwrap_or_default()
                        .into_iter()
                        .map(DependencyGroupSpecification::String),
                );
        }

        match dependency_groups_strategy {
            // When using `SetDefaultGroups` strategy, all non-optional dependency groups are
            // referenced in `default-groups` under `[tool.uv]` section. If we only have `dev`
            // dependency group, do not set `default-groups`, as this is already uv's default.
            DependencyGroupsStrategy::SetDefaultGroups => {
                if !dependency_groups.keys().eq(["dev"]) {
                    default_groups.extend(
                        dependency_groups
                            .keys()
                            .filter(|&group| !optional_groups.contains(group))
                            .map(ToString::to_string),
                    );
                }
            }
            // When using `IncludeInDev` strategy, non-optional dependency groups (except `dev` one)
            // are referenced from `dev` dependency group with `{ include-group = "<group>" }`.
            DependencyGroupsStrategy::IncludeInDev => {
                dependency_groups
                    .entry("dev".to_string())
                    .or_default()
                    .extend(
                        poetry_group
                            .keys()
                            .filter(|&k| k != "dev" && !optional_groups.contains(k))
                            .map(|g| DependencyGroupSpecification::Map {
                                include_group: Some(g.clone()),
                            }),
                    );
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
