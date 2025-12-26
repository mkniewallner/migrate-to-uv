use crate::converters::poetry::build_backend;
use crate::converters::poetry::build_backend::{
    get_include_distribution_format, get_packages_distribution_format,
};
use crate::schema::hatch::{Build, BuildTarget, Hatch};
use crate::schema::poetry::{Include, Package};
use indexmap::IndexMap;
use owo_colors::OwoColorize;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

#[derive(Default)]
struct HatchTargetsInclude {
    sdist_include: Option<Vec<String>>,
    wheel_include: Option<Vec<String>>,
    sdist_force_include: Option<IndexMap<String, String>>,
    wheel_force_include: Option<IndexMap<String, String>>,
    wheel_sources: Option<IndexMap<String, String>>,
}

impl HatchTargetsInclude {
    pub fn new(
        sdist_include: Vec<String>,
        wheel_include: Vec<String>,
        sdist_force_include: IndexMap<String, String>,
        wheel_force_include: IndexMap<String, String>,
        wheel_sources: IndexMap<String, String>,
    ) -> Self {
        Self {
            sdist_include: build_backend::non_empty_vec(sdist_include),
            wheel_include: build_backend::non_empty_vec(wheel_include),
            sdist_force_include: build_backend::non_empty_index_map(sdist_force_include),
            wheel_force_include: build_backend::non_empty_index_map(wheel_force_include),
            wheel_sources: build_backend::non_empty_index_map(wheel_sources),
        }
    }
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
pub fn get_build_backend(
    project_path: &Path,
    packages: Option<&Vec<Package>>,
    include: Option<&Vec<Include>>,
    exclude: Option<&Vec<String>>,
) -> Result<Option<Hatch>, Vec<String>> {
    let mut errors = Vec::new();

    let mut targets = IndexMap::new();
    let hatch_targets_include = match get_include(project_path, packages, include) {
        Ok(hatch_targets_include) => hatch_targets_include,
        Err(e) => {
            errors.extend(e);
            HatchTargetsInclude::default()
        }
    };

    let sdist_target = BuildTarget {
        include: hatch_targets_include.sdist_include,
        force_include: hatch_targets_include.sdist_force_include,
        exclude: exclude.cloned(),
        sources: None,
    };
    let wheel_target = BuildTarget {
        include: hatch_targets_include.wheel_include,
        force_include: hatch_targets_include.wheel_force_include,
        exclude: exclude.cloned(),
        sources: hatch_targets_include.wheel_sources,
    };

    if sdist_target != BuildTarget::default() {
        targets.insert("sdist".to_string(), sdist_target);
    }
    if wheel_target != BuildTarget::default() {
        targets.insert("wheel".to_string(), wheel_target);
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    if targets.is_empty() {
        return Ok(None);
    }

    Ok(Some(Hatch {
        build: Some(Build {
            targets: Some(targets),
        }),
    }))
}

/// Inclusion behavior: <https://hatch.pypa.io/latest/config/build/#patterns>
/// Path rewriting behavior: <https://hatch.pypa.io/latest/config/build/#rewriting-paths>
fn get_include(
    project_path: &Path,
    packages: Option<&Vec<Package>>,
    include: Option<&Vec<Include>>,
) -> Result<HatchTargetsInclude, Vec<String>> {
    let mut sdist_include = Vec::new();
    let mut wheel_include = Vec::new();
    let mut sdist_force_include = IndexMap::new();
    let mut wheel_force_include = IndexMap::new();
    let mut wheel_sources = IndexMap::new();

    let mut errors = Vec::new();

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

            let (add_to_sdist, add_to_wheel) = get_packages_distribution_format(format.as_ref());

            if add_to_sdist {
                sdist_include.push(include_with_from.clone());
            }

            if add_to_wheel {
                wheel_include.push(include_with_from.clone());

                match get_source(
                    project_path,
                    include.clone(),
                    include_with_from,
                    to.as_ref(),
                    from.as_ref(),
                ) {
                    Ok(Some((from, to))) => {
                        wheel_sources.insert(from, to);
                    }
                    Err(e) => errors.push(e),
                    Ok(None) => (),
                }
            }
        }
    }

    // https://python-poetry.org/docs/pyproject/#exclude-and-include
    if let Some(include) = include {
        for inc in include {
            let (path, format) = match inc {
                Include::Map { path, format } => (path, format.as_ref()),
                Include::String(path) => (path, None),
            };
            let (add_to_sdist, add_to_wheel) = get_include_distribution_format(format);

            if add_to_sdist {
                sdist_force_include.insert(path.clone(), path.clone());
            }

            if add_to_wheel {
                wheel_force_include.insert(path.clone(), path.clone());
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(HatchTargetsInclude::new(
        sdist_include,
        wheel_include,
        sdist_force_include,
        wheel_force_include,
        wheel_sources,
    ))
}

/// Get hatch source, to rewrite path from a directory to another directory in the built artifact.
/// <https://hatch.pypa.io/latest/config/build/#rewriting-paths>
fn get_source(
    project_path: &Path,
    include: String,
    include_with_from: String,
    to: Option<&String>,
    from: Option<&String>,
) -> Result<Option<(String, String)>, String> {
    let project_include_with_from = project_path.join(&include_with_from);

    // Rewrite for files cannot be handled properly, since Hatch only rewrites directories.
    // We could take the parent directory in that case, but this could conflict with other
    // rules in place.
    if from.is_some() && !include.contains('*') && project_include_with_from.is_file() {
        return Err(format!(
            "\"{}\" from \"{}\" cannot be converted to Hatch, as it uses \"{}\" on a file, which cannot be expressed with Hatch.",
            include.bold(),
            "poetry.packages.include".bold(),
            "from".bold(),
        ));
    }

    if let Some(to) = to {
        return if include.contains('*') {
            // Hatch path rewrite behaves differently to Poetry, as rewriting is only possible on
            // static paths, so we build the longest path until we reach a glob for both the initial
            // and the path to rewrite to, to only rewrite the static part for both.
            let from_without_glob = extract_parent_path_from_glob(&include_with_from).unwrap();
            let to_without_glob = extract_parent_path_from_glob(&include).unwrap();

            Ok(Some((
                from_without_glob,
                Path::new(to)
                    .join(to_without_glob)
                    .display()
                    .to_string()
                    // Ensure that separator remains "/" (Windows uses "\").
                    .replace(MAIN_SEPARATOR, "/"),
            )))
        } else {
            Ok(Some((
                PathBuf::from(include_with_from)
                    .display()
                    .to_string()
                    .replace(MAIN_SEPARATOR, "/"),
                PathBuf::from(to)
                    .join(&include)
                    .display()
                    .to_string()
                    .replace(MAIN_SEPARATOR, "/"),
            )))
        };
    }

    if from.is_some() {
        return Ok(Some((include_with_from, include)));
    }

    Ok(None)
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
