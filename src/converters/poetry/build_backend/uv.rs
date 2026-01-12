use crate::converters::poetry::build_backend::{
    get_include_distribution_format, get_packages_distribution_format,
};
use crate::schema::poetry::{Include, Package};
use crate::schema::utils::SingleOrVec;
use crate::schema::uv::UvBuildBackend;
use crate::utils::non_empty_vec;
use owo_colors::OwoColorize;
use std::path::Path;

pub fn get_build_backend(
    project_path: &Path,
    packages: Option<&Vec<Package>>,
    include: Option<&Vec<Include>>,
    exclude: Option<&Vec<String>>,
) -> Result<Option<UvBuildBackend>, Vec<String>> {
    let mut errors = Vec::new();

    let mut module_name: Vec<String> = Vec::new();
    let mut source_include: Vec<String> = Vec::new();
    let mut source_exclude: Vec<String> = Vec::new();
    let mut wheel_exclude: Vec<String> = Vec::new();

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

            let (add_to_sdist, add_to_wheel) = get_packages_distribution_format(format.as_ref());

            let contains_glob = include.contains('*');
            let is_file = project_path.join(include).is_file();

            if contains_glob || is_file {
                let reason = if contains_glob {
                    "uses globs"
                } else {
                    "is a file"
                };

                if add_to_sdist && add_to_wheel {
                    errors.push(
                        format!(
                            "\"{}\" from \"{}\" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, and {}, which cannot be expressed with uv.",
                            include.bold(),
                            "poetry.packages.include".bold(),
                            reason,
                        )
                    );
                } else if add_to_sdist {
                    source_include.push(include.clone());
                } else if add_to_wheel {
                    errors.push(
                        format!(
                            "\"{}\" from \"{}\" cannot be converted to uv, as it is configured to be added to wheels only, and {}, which cannot be expressed with uv.",
                            include.bold(),
                            "poetry.packages.include".bold(),
                            reason,
                        )
                    );
                }
            } else {
                let name = include.replace('/', ".");
                let has_init_file = has_init_file(project_path, include, from.as_ref());

                if !has_init_file {
                    errors.push(
                        format!(
                            "\"{}\" from \"{}\" cannot be converted to uv, as it does not contain an \"{}\" file, which is required by uv for packages.",
                            include.bold(),
                            "poetry.packages.include".bold(),
                            "__init__.py".bold(),
                        )
                    );
                }

                if add_to_sdist && add_to_wheel {
                    if has_init_file {
                        module_name.push(name.clone());
                    }
                } else if add_to_sdist {
                    if has_init_file {
                        module_name.push(name.clone());
                        wheel_exclude.push(include.clone());
                    }
                } else if add_to_wheel {
                    errors.push(
                        format!(
                            "\"{}\" from \"{}\" cannot be converted to uv, as it is configured to be added to wheels only, which cannot be expressed with uv.",
                            include.bold(),
                            "poetry.packages.include".bold(),
                        )
                    );
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

            if add_to_sdist && add_to_wheel {
                errors.push(
                    format!(
                        "\"{}\" from \"{}\" cannot be converted to uv, as it is configured to be added to both source distribution and wheels, which cannot be expressed with uv.",
                        path.bold(),
                        "poetry.include".bold(),
                    )
                );
            } else if add_to_sdist {
                if path.contains('*') || project_path.join(path).is_file() {
                    source_include.push(path.clone());
                } else {
                    source_include.push(format!("{path}/**"));
                }
            } else if add_to_wheel {
                errors.push(
                    format!(
                        "\"{}\" from \"{}\" cannot be converted to uv, as it is configured to be added to wheels only, which cannot be expressed with uv.",
                        path.bold(),
                        "poetry.include".bold(),
                    )
                );
            }
        }
    }

    // https://python-poetry.org/docs/pyproject/#exclude-and-include
    if let Some(exclude) = exclude {
        source_exclude.extend(exclude.clone());
        wheel_exclude.extend(exclude.clone());
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
        source_include: non_empty_vec(source_include),
        source_exclude: non_empty_vec(source_exclude),
        wheel_exclude: non_empty_vec(wheel_exclude),
        ..UvBuildBackend::default()
    }))
}

fn has_init_file(project_path: &Path, include: &String, from: Option<&String>) -> bool {
    let path = from.map_or_else(
        || project_path.join(include).join("__init__.py"),
        |from| project_path.join(from).join(include).join("__init__.py"),
    );

    path.exists()
}
