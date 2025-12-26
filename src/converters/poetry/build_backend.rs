use crate::converters::BuildBackend;
use crate::schema::hatch::{Build, BuildTarget, Hatch};
use crate::schema::poetry::{Format, Include, Package};
use crate::schema::pyproject::BuildSystem;
use crate::schema::utils::SingleOrVec;
use crate::schema::uv::UvBuildBackend;
use indexmap::IndexMap;
use owo_colors::OwoColorize;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

type HatchTargetsIncludeAndSource = (
    Option<Vec<String>>,
    Option<Vec<String>>,
    Option<IndexMap<String, String>>,
    Option<IndexMap<String, String>>,
    Option<IndexMap<String, String>>,
);

pub fn get_new_build_system(
    current_build_system: Option<BuildSystem>,
    new_build_system: Option<BuildBackend>,
) -> Option<BuildSystem> {
    if current_build_system?.build_backend? == "poetry.core.masonry.api" {
        return match new_build_system {
            None | Some(BuildBackend::Hatch) => Some(BuildSystem {
                requires: vec!["hatchling".to_string()],
                build_backend: Some("hatchling.build".to_string()),
            }),
            Some(BuildBackend::Uv) => Some(BuildSystem {
                requires: vec!["uv_build".to_string()],
                build_backend: Some("uv_build".to_string()),
            }),
        };
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
    project_path: &Path,
    packages: Option<&Vec<Package>>,
    include: Option<&Vec<Include>>,
    exclude: Option<&Vec<String>>,
) -> Result<Option<Hatch>, Vec<String>> {
    let errors = Vec::new();

    let mut targets = IndexMap::new();
    let (sdist_include, wheel_include, sdist_force_include, wheel_force_include, wheel_sources) =
        get_hatch_include(project_path, packages, include);

    let sdist_target = BuildTarget {
        include: sdist_include,
        force_include: sdist_force_include,
        exclude: exclude.cloned(),
        sources: None,
    };
    let wheel_target = BuildTarget {
        include: wheel_include,
        force_include: wheel_force_include,
        exclude: exclude.cloned(),
        sources: wheel_sources,
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
fn get_hatch_include(
    project_path: &Path,
    packages: Option<&Vec<Package>>,
    include: Option<&Vec<Include>>,
) -> HatchTargetsIncludeAndSource {
    let mut sdist_include = Vec::new();
    let mut wheel_include = Vec::new();
    let mut sdist_force_include = IndexMap::new();
    let mut wheel_force_include = IndexMap::new();
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

            let (add_to_sdist, add_to_wheel) = match format {
                None => (true, true),
                Some(SingleOrVec::Single(Format::Sdist)) => (true, false),
                Some(SingleOrVec::Single(Format::Wheel)) => (false, true),
                // Note: An empty `format = []` in Poetry means that the files will not be added to
                // any distribution at all.
                Some(SingleOrVec::Vec(vec)) => {
                    (vec.contains(&Format::Sdist), vec.contains(&Format::Wheel))
                }
            };

            if add_to_sdist {
                sdist_include.push(include_with_from.clone());
            }

            if add_to_wheel {
                wheel_include.push(include_with_from.clone());

                if let Some((from, to)) = get_hatch_source(
                    project_path,
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

    // https://python-poetry.org/docs/pyproject/#exclude-and-include
    if let Some(include) = include {
        for inc in include {
            let (add_to_sdist, add_to_wheel, path) = match inc {
                Include::String(path)
                | Include::Map {
                    path,
                    format: None | Some(SingleOrVec::Single(Format::Sdist)),
                } => {
                    // https://python-poetry.org/docs/1.8/pyproject/#include-and-exclude
                    // If there is no format specified, files are only added to sdist.
                    (true, false, path)
                }
                Include::Map {
                    path,
                    format: Some(SingleOrVec::Single(Format::Wheel)),
                } => (false, true, path),
                // Note: An empty `format = []` in Poetry means that the files will not be added to
                // any distribution at all.
                Include::Map {
                    path,
                    format: Some(SingleOrVec::Vec(format)),
                } => match format[..] {
                    [Format::Sdist, Format::Wheel] => (true, true, path),
                    [Format::Sdist] => (true, false, path),
                    [Format::Wheel] => (false, true, path),
                    _ => (false, false, path),
                },
            };

            if add_to_sdist {
                sdist_force_include.insert(path.clone(), path.clone());
            }

            if add_to_wheel {
                wheel_force_include.insert(path.clone(), path.clone());
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
        if sdist_force_include.is_empty() {
            None
        } else {
            Some(sdist_force_include)
        },
        if wheel_force_include.is_empty() {
            None
        } else {
            Some(wheel_force_include)
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
    _project_path: &Path,
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

pub fn get_uv(
    project_path: &Path,
    packages: Option<&Vec<Package>>,
    include: Option<&Vec<Include>>,
    exclude: Option<&Vec<String>>,
) -> Result<Option<UvBuildBackend>, Vec<String>> {
    let mut errors = Vec::new();

    let mut module_name = Vec::new();
    let mut source_include = Vec::new();
    let mut source_exclude = Vec::new();
    let mut wheel_exclude = Vec::new();

    // https://python-poetry.org/docs/pyproject/#packages
    if let Some(packages) = packages {
        for Package {
            include,
            format,
            from,
            to,
        } in packages
        {
            if from.is_some() {
                errors.push(
                    format!(
                        "\"{}\" from \"{}\" cannot be converted to uv, as it uses \"{}\", which cannot be expressed with uv.",
                        include.bold(),
                        "poetry.packages.include".bold(),
                        "from".bold(),
                    )
                );
            }

            if to.is_some() {
                errors.push(
                    format!(
                        "\"{}\" from \"{}\" cannot be converted to uv, as it uses \"{}\", which cannot be expressed with uv.",
                        include.bold(),
                        "poetry.packages.include".bold(),
                        "to".bold(),
                    )
                );
            }

            let contains_glob = include.contains('*');
            let is_file = project_path.join(include).is_file();

            if contains_glob || is_file {
                let reason = if contains_glob {
                    "uses globs"
                } else {
                    "is a file"
                };

                match format {
                    None => {
                        errors.push(
                            format!(
                                "\"{}\" from \"{}\" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, and {}, which cannot be expressed with uv.",
                                include.bold(),
                                "poetry.packages.include".bold(),
                                reason,
                            )
                        );
                    }
                    Some(SingleOrVec::Single(Format::Wheel)) => {
                        errors.push(
                            format!(
                                "\"{}\" from \"{}\" cannot be converted to uv, as it is configured to be added to wheels only, and {}, which cannot be expressed with uv.",
                                include.bold(),
                                "poetry.packages.include".bold(),
                                reason,
                            )
                        );
                    }
                    Some(SingleOrVec::Single(Format::Sdist)) => {
                        source_include.push(include.clone());
                    }
                    Some(SingleOrVec::Vec(vec)) => {
                        // Note: An empty `format = []` in Poetry means that the files will not be added to
                        // any distribution at all.
                        if !vec.is_empty() {
                            if vec.contains(&Format::Sdist) && !vec.contains(&Format::Wheel) {
                                source_include.push(include.clone());
                            } else if vec.contains(&Format::Wheel) && vec.contains(&Format::Sdist) {
                                errors.push(
                                    format!(
                                        "\"{}\" from \"{}\" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, and {}, which cannot be expressed with uv.",
                                        include.bold(),
                                        "poetry.packages.include".bold(),
                                        reason,
                                    )
                                );
                            } else if vec.contains(&Format::Wheel) && !vec.contains(&Format::Sdist)
                            {
                                errors.push(
                                    format!(
                                        "\"{}\" from \"{}\" cannot be converted to uv, as it is configured to be added to wheels only, and {}, which cannot be expressed with uv.",
                                        include.bold(),
                                        "poetry.packages.include".bold(),
                                        reason,
                                    )
                                );
                            }
                        }
                    }
                }
            } else {
                let name = include.replace('/', ".");

                match format {
                    None => {
                        if has_init_file(project_path, include, from.as_ref(), &mut errors) {
                            module_name.push(name.clone());
                        }
                    }
                    Some(SingleOrVec::Single(Format::Wheel)) => {
                        errors.push(
                            format!(
                                "\"{}\" from \"{}\" cannot be converted to uv, as it is configured to be added to wheels only, which cannot be expressed with uv.",
                                include.bold(),
                                "poetry.packages.include".bold(),
                            )
                        );
                    }
                    Some(SingleOrVec::Single(Format::Sdist)) => {
                        if has_init_file(project_path, include, from.as_ref(), &mut errors) {
                            module_name.push(name.clone());
                            wheel_exclude.push(include.clone());
                        }
                    }

                    Some(SingleOrVec::Vec(vec)) => {
                        // Note: An empty `format = []` in Poetry means that the files will not be added to
                        // any distribution at all.
                        if !vec.is_empty() {
                            if vec.contains(&Format::Sdist) && vec.contains(&Format::Wheel) {
                                if has_init_file(project_path, include, from.as_ref(), &mut errors)
                                {
                                    module_name.push(name.clone());
                                }
                            } else if vec.contains(&Format::Sdist) && !vec.contains(&Format::Wheel)
                            {
                                if has_init_file(project_path, include, from.as_ref(), &mut errors)
                                {
                                    module_name.push(name.clone());
                                    wheel_exclude.push(include.clone());
                                }
                            } else if vec.contains(&Format::Wheel)
                                && !vec.contains(&Format::Sdist)
                                && has_init_file(project_path, include, from.as_ref(), &mut errors)
                            {
                                module_name.push(name.clone());
                                source_exclude.push(include.clone());
                            }
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
                Include::String(path)
                | Include::Map {
                    path,
                    format: None | Some(SingleOrVec::Single(Format::Sdist)),
                } => {
                    // https://python-poetry.org/docs/1.8/pyproject/#include-and-exclude
                    // If there is no format specified, files are only added to sdist.
                    if path.contains('*') || project_path.join(path).is_file() {
                        source_include.push(path.clone());
                    } else {
                        source_include.push(format!("{path}/**"));
                    }
                }
                Include::Map {
                    path,
                    format: Some(SingleOrVec::Single(Format::Wheel)),
                } => {
                    errors.push(
                        format!(
                            "\"{}\" from \"{}\" cannot be converted to uv, as it is configured to be added to wheels only, which cannot be expressed with uv.",
                            path.bold(),
                            "poetry.include".bold(),
                        )
                    );
                }
                // Note: An empty `format = []` in Poetry means that the files will not be added to
                // any distribution at all.
                Include::Map {
                    path,
                    format: Some(SingleOrVec::Vec(format)),
                } => match format[..] {
                    [Format::Sdist, Format::Wheel] => {
                        errors.push(
                            format!(
                                "\"{}\" from \"{}\" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, which cannot be expressed with uv.",
                                path.bold(),
                                "poetry.include".bold(),
                            )
                        );
                    }
                    [Format::Sdist] => {
                        if path.contains('*') || project_path.join(path).is_file() {
                            source_include.push(path.clone());
                        } else {
                            source_include.push(format!("{path}/**"));
                        }
                    }
                    [Format::Wheel] => {
                        errors.push(
                            format!(
                                "\"{}\" from \"{}\" cannot be converted to uv, as it is configured to be added to wheels only, which cannot be expressed with uv.",
                                path.bold(),
                                "poetry.include".bold(),
                            )
                        );
                    }
                    _ => (),
                },
            }
        }
    }

    // https://python-poetry.org/docs/pyproject/#exclude-and-include
    if let Some(exclude) = exclude {
        for excl in exclude {
            source_exclude.push(excl.clone());
            wheel_exclude.push(excl.clone());
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    if module_name.is_empty()
        && source_include.is_empty()
        && source_exclude.is_empty()
        && wheel_exclude.is_empty()
    {
        return Ok(None);
    }

    Ok(Some(UvBuildBackend {
        module_name: Some(SingleOrVec::Vec(module_name)),
        // By default, uv expects the modules to be in a "src" directory. Since Poetry does not
        // provide a similar option, we want to default to the same thing as Poetry, i.e. an empty
        // string.
        module_root: Some(String::new()),
        source_include: Some(source_include),
        source_exclude: Some(source_exclude),
        wheel_exclude: Some(wheel_exclude),
        ..UvBuildBackend::default()
    }))
}

fn has_init_file(
    project_path: &Path,
    include: &String,
    from: Option<&String>,
    errors: &mut Vec<String>,
) -> bool {
    let path = if let Some(from) = from {
        project_path.join(from).join(include).join("__init__.py")
    } else {
        project_path.join(include).join("__init__.py")
    };

    if path.exists() {
        true
    } else {
        errors.push(
            format!(
                "\"{}\" from \"{}\" cannot be converted to uv, as it does not contain an \"{}\" file, which is required by uv for packages.",
                include.bold(),
                "poetry.packages.include".bold(),
                "__init__.py".bold(),
            )
        );
        false
    }
}
