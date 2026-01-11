use crate::converters::{BuildBackend, ConverterOptions};
use crate::errors::{add_recoverable_error, add_unrecoverable_error};
use crate::schema::hatch::Hatch;
use crate::schema::poetry::{Format, Poetry};
use crate::schema::pyproject::BuildSystem;
use crate::schema::utils::SingleOrVec;
use crate::schema::uv::UvBuildBackend;
use indexmap::IndexMap;
use owo_colors::OwoColorize;
use std::fmt::Display;

pub mod hatch;
pub mod uv;

pub enum BuildBackendObject {
    Uv(UvBuildBackend),
    Hatch(Hatch),
}

impl Display for BuildBackendObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uv(_) => write!(f, "uv"),
            Self::Hatch(_) => write!(f, "Hatch"),
        }
    }
}

pub fn get_new_build_system(
    current_build_system: Option<BuildSystem>,
    keep_current_build_backend: bool,
    new_build_system: Option<&BuildBackendObject>,
) -> Option<BuildSystem> {
    if keep_current_build_backend {
        return None;
    }

    if current_build_system?.build_backend? == "poetry.core.masonry.api" {
        return match new_build_system {
            None | Some(BuildBackendObject::Uv(_)) => Some(BuildSystem {
                requires: vec!["uv_build".to_string()],
                build_backend: Some("uv_build".to_string()),
            }),
            Some(BuildBackendObject::Hatch(_)) => Some(BuildSystem {
                requires: vec!["hatchling".to_string()],
                build_backend: Some("hatchling.build".to_string()),
            }),
        };
    }

    None
}

/// Get build backend based on converter options. If `--build-backend` is not set or set to `hatch`,
/// Hatch is selected. If `--build-backend` is set to `uv`, Uv is selected.
pub fn get_build_backend(
    converter_options: &ConverterOptions,
    poetry: &Poetry,
) -> Option<BuildBackendObject> {
    if converter_options.keep_current_build_backend {
        return None;
    }

    match &converter_options.build_backend {
        None => {
            let uv = uv::get_build_backend(
                &converter_options.project_path,
                poetry.packages.as_ref(),
                poetry.include.as_ref(),
                poetry.exclude.as_ref(),
            );

            match uv {
                Ok(Some(uv)) => Some(BuildBackendObject::Uv(uv)),
                Err(_) => {
                    add_recoverable_error(
                        "Migrating build backend to Hatch because package distribution metadata is too complex for uv.".to_string()
                    );

                    let hatch = hatch::get_build_backend(
                        &converter_options.project_path,
                        poetry.packages.as_ref(),
                        poetry.include.as_ref(),
                        poetry.exclude.as_ref(),
                    );

                    match hatch {
                        Ok(Some(hatch)) => Some(BuildBackendObject::Hatch(hatch)),
                        Err(errors) => {
                            for error in errors {
                                add_unrecoverable_error(error.clone());
                            }
                            None
                        }
                        Ok(None) => None,
                    }
                }
                Ok(None) => None,
            }
        }
        Some(BuildBackend::Uv) => {
            let uv = uv::get_build_backend(
                &converter_options.project_path,
                poetry.packages.as_ref(),
                poetry.include.as_ref(),
                poetry.exclude.as_ref(),
            );

            match uv {
                Ok(Some(uv)) => Some(BuildBackendObject::Uv(uv)),
                Err(errors) => {
                    for error in errors {
                        add_unrecoverable_error(error.clone());
                    }

                    add_unrecoverable_error(format!(
                        "Package distribution cound not be migrated to uv build backend due to the issues above. Consider using Hatch build backend with \"{}\".",
                        "--build-backend hatch".bold(),
                    ));

                    None
                }
                Ok(None) => None,
            }
        }
        Some(BuildBackend::Hatch) => {
            let hatch = hatch::get_build_backend(
                &converter_options.project_path,
                poetry.packages.as_ref(),
                poetry.include.as_ref(),
                poetry.exclude.as_ref(),
            );

            match hatch {
                Ok(Some(hatch)) => Some(BuildBackendObject::Hatch(hatch)),
                Err(errors) => {
                    for error in errors {
                        add_unrecoverable_error(error.clone());
                    }

                    None
                }
                Ok(None) => None,
            }
        }
    }
}

/// Get the distributions to include an item from `packages` to.
/// <https://python-poetry.org/docs/pyproject/#packages>
fn get_packages_distribution_format(format: Option<&SingleOrVec<Format>>) -> (bool, bool) {
    match format {
        None => (true, true),
        Some(SingleOrVec::Single(Format::Sdist)) => (true, false),
        Some(SingleOrVec::Single(Format::Wheel)) => (false, true),
        // Note: An empty `format = []` in Poetry means that the files will not be added to
        // any distribution at all.
        Some(SingleOrVec::Vec(vec)) => (vec.contains(&Format::Sdist), vec.contains(&Format::Wheel)),
    }
}

/// Get the distributions to include an item from `include` to.
/// <https://python-poetry.org/docs/pyproject/#exclude-and-include>
fn get_include_distribution_format(format: Option<&SingleOrVec<Format>>) -> (bool, bool) {
    match format {
        // If there is no format specified, files are only added to sdist.
        None | Some(SingleOrVec::Single(Format::Sdist)) => (true, false),
        Some(SingleOrVec::Single(Format::Wheel)) => (false, true),
        // Note: An empty `format = []` in Poetry means that the files will not be added to
        // any distribution at all.
        Some(SingleOrVec::Vec(vec)) => (vec.contains(&Format::Sdist), vec.contains(&Format::Wheel)),
    }
}

fn non_empty_vec<T>(vec: Vec<T>) -> Option<Vec<T>> {
    if vec.is_empty() {
        return None;
    }
    Some(vec)
}

fn non_empty_index_map<T, U>(map: IndexMap<T, U>) -> Option<IndexMap<T, U>> {
    if map.is_empty() {
        return None;
    }
    Some(map)
}
