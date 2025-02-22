use crate::schema::hatch::{Build, BuildTarget, Hatch};
use crate::schema::poetry::{Format, Include, Package};
use crate::schema::pyproject::BuildSystem;
use crate::schema::utils::SingleOrVec;
use indexmap::IndexMap;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

type HatchTargetsIncludeAndSource = (
    Option<Vec<String>>,
    Option<Vec<String>>,
    Option<IndexMap<String, String>>,
);

pub fn get_new_build_system(build_system: Option<BuildSystem>) -> Option<BuildSystem> {
    if build_system?.build_backend? == "poetry.core.masonry.api" {
        return Some(BuildSystem {
            requires: vec!["hatchling".to_string()],
            build_backend: Some("hatchling.build".to_string()),
        });
    }
    None
}

/// Construct hatch package metadata (<https://hatch.pypa.io/latest/config/build/>) from Poetry
/// `packages` (<https://python-poetry.org/docs/pyproject/#packages>) and `include`/`exclude`
/// (<https://python-poetry.org/docs/pyproject/#exclude-and-include>).
///
/// Poetry `packages` and `include` are converted to hatch `include`.
///
/// If a pattern in `packages` uses `to`, an entry is populated in `sources` under hatch `wheel`
/// target to rewrite the path the same way as Poetry does in wheels. Note that although Poetry's
/// documentation does not specify it, `to` only rewrites paths in wheels, and not sdist, so we only
/// apply path rewriting in `wheel` target.
///
/// Poetry `exclude` is converted as is to hatch `exclude`.
///
pub fn get_hatch(
    packages: Option<&Vec<Package>>,
    include: Option<&Vec<Include>>,
    exclude: Option<&Vec<String>>,
) -> Option<Hatch> {
    let mut targets = IndexMap::new();
    let (sdist_include, wheel_include, wheel_sources) = get_hatch_include(packages, include);

    let sdist_target = BuildTarget {
        include: sdist_include,
        exclude: exclude.cloned(),
        sources: None,
    };
    let wheel_target = BuildTarget {
        include: wheel_include,
        exclude: exclude.cloned(),
        sources: wheel_sources,
    };

    if sdist_target != BuildTarget::default() {
        targets.insert("sdist".to_string(), sdist_target);
    }
    if wheel_target != BuildTarget::default() {
        targets.insert("wheel".to_string(), wheel_target);
    }

    if targets.is_empty() {
        return None;
    }

    Some(Hatch {
        build: Some(Build {
            targets: Some(targets),
        }),
    })
}

/// Inclusion behavior: <https://hatch.pypa.io/latest/config/build/#patterns>
/// Path rewriting behavior: <https://hatch.pypa.io/latest/config/build/#rewriting-paths>
fn get_hatch_include(
    packages: Option<&Vec<Package>>,
    include: Option<&Vec<Include>>,
) -> HatchTargetsIncludeAndSource {
    let mut sdist_include = Vec::new();
    let mut wheel_include = Vec::new();
    let mut wheel_sources = IndexMap::new();

    // https://python-poetry.org/docs/pyproject/#packages
    if let Some(packages) = packages {
        for Package {
            include,
            format,
            from,
            to,
        } in packages
        {
            let include_with_from = PathBuf::from(from.as_ref().map_or("", |from| from))
                .join(include)
                .display()
                .to_string()
                // Ensure that separator remains "/" (Windows uses "\").
                .replace(MAIN_SEPARATOR, "/");

            match format {
                None => {
                    sdist_include.push(include_with_from.clone());
                    wheel_include.push(include_with_from.clone());

                    if let Some((from, to)) = get_hatch_source(
                        include.clone(),
                        include_with_from.clone(),
                        to.as_ref(),
                        from.as_ref(),
                    ) {
                        wheel_sources.insert(from, to);
                    }
                }
                Some(SingleOrVec::Single(Format::Sdist)) => {
                    sdist_include.push(include_with_from.clone());
                }
                Some(SingleOrVec::Single(Format::Wheel)) => {
                    wheel_include.push(include_with_from.clone());

                    if let Some((from, to)) = get_hatch_source(
                        include.clone(),
                        include_with_from.clone(),
                        to.as_ref(),
                        from.as_ref(),
                    ) {
                        wheel_sources.insert(from, to);
                    }
                }
                Some(SingleOrVec::Vec(vec)) => {
                    if vec.contains(&Format::Sdist) || vec.is_empty() {
                        sdist_include.push(include_with_from.clone());
                    }
                    if vec.contains(&Format::Wheel) || vec.is_empty() {
                        wheel_include.push(include_with_from.clone());

                        if let Some((from, to)) = get_hatch_source(
                            include.clone(),
                            include_with_from,
                            to.as_ref(),
                            from.as_ref(),
                        ) {
                            wheel_sources.insert(from, to);
                        }
                    }
                }
            }
        }
    }

    // https://python-poetry.org/docs/pyproject/#exclude-and-include
    if let Some(include) = include {
        for inc in include {
            match inc {
                Include::String(path) | Include::Map { path, format: None } => {
                    sdist_include.push(path.to_string());
                    wheel_include.push(path.to_string());
                }
                Include::Map {
                    path,
                    format: Some(SingleOrVec::Vec(format)),
                } => match format[..] {
                    [] | [Format::Sdist, Format::Wheel] => {
                        sdist_include.push(path.to_string());
                        wheel_include.push(path.to_string());
                    }
                    [Format::Sdist] => sdist_include.push(path.to_string()),
                    [Format::Wheel] => wheel_include.push(path.to_string()),
                    _ => (),
                },
                Include::Map {
                    path,
                    format: Some(SingleOrVec::Single(Format::Sdist)),
                } => sdist_include.push(path.to_string()),
                Include::Map {
                    path,
                    format: Some(SingleOrVec::Single(Format::Wheel)),
                } => wheel_include.push(path.to_string()),
            }
        }
    }

    (
        if sdist_include.is_empty() {
            None
        } else {
            Some(sdist_include)
        },
        if wheel_include.is_empty() {
            None
        } else {
            Some(wheel_include)
        },
        if wheel_sources.is_empty() {
            None
        } else {
            Some(wheel_sources)
        },
    )
}

/// Get hatch source, to rewrite path from a directory to another directory in the built artifact.
/// <https://hatch.pypa.io/latest/config/build/#rewriting-paths>
fn get_hatch_source(
    include: String,
    include_with_from: String,
    to: Option<&String>,
    from: Option<&String>,
) -> Option<(String, String)> {
    if let Some(to) = to {
        return if include.contains('*') {
            // Hatch path rewrite behaves differently to Poetry, as rewriting is only possible on
            // static paths, so we build the longest path until we reach a glob for both the initial
            // and the path to rewrite to, to only rewrite the static part for both.
            let from_without_glob = extract_parent_path_from_glob(&include_with_from)?;
            let to_without_glob = extract_parent_path_from_glob(&include)?;

            Some((
                from_without_glob,
                Path::new(to)
                    .join(to_without_glob)
                    .display()
                    .to_string()
                    // Ensure that separator remains "/" (Windows uses "\").
                    .replace(MAIN_SEPARATOR, "/"),
            ))
        } else {
            Some((
                include_with_from,
                Path::new(to)
                    .join(include)
                    .display()
                    .to_string()
                    // Ensure that separator remains "/" (Windows uses "\").
                    .replace(MAIN_SEPARATOR, "/"),
            ))
        };
    }

    if from.is_some() {
        return Some((include_with_from, include));
    }

    None
}

/// Extract the longest path part from a path until a glob is found.
fn extract_parent_path_from_glob(s: &str) -> Option<String> {
    let mut parents = Vec::new();

    for part in s.split('/') {
        if part.contains('*') {
            break;
        }
        parents.push(part);
    }

    if parents.is_empty() {
        return None;
    }
    Some(parents.join("/"))
}
