use crate::schema::poetry::{Format, Include, Package};
use crate::schema::utils::SingleOrVec;
use crate::schema::uv::UvBuildBackend;
use owo_colors::OwoColorize;
use std::path::Path;

pub fn get_build_backend(
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
        source_include: if source_include.is_empty() {
            None
        } else {
            Some(source_include)
        },
        source_exclude: if source_exclude.is_empty() {
            None
        } else {
            Some(source_exclude)
        },
        wheel_exclude: if wheel_exclude.is_empty() {
            None
        } else {
            Some(wheel_exclude)
        },
        ..UvBuildBackend::default()
    }))
}

fn has_init_file(
    project_path: &Path,
    include: &String,
    from: Option<&String>,
    errors: &mut Vec<String>,
) -> bool {
    let path = from.map_or_else(
        || project_path.join(include).join("__init__.py"),
        |from| project_path.join(from).join(include).join("__init__.py"),
    );

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
