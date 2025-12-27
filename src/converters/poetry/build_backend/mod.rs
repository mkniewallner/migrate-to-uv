use crate::converters::{BuildBackend, ConverterOptions};
use crate::errors::add_unrecoverable_error;
use crate::schema::hatch::Hatch;
use crate::schema::poetry::Poetry;
use crate::schema::pyproject::BuildSystem;
use crate::schema::uv::UvBuildBackend;
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
            Self::Hatch(_) => write!(f, "hatch"),
        }
    }
}

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

/// Get build backend based on converter options. If `--build-backend` is not set or set to `hatch`,
/// Hatch is selected. If `--build-backend` is set to `uv`, Uv is selected.
pub fn get_build_backend(
    converter_options: &ConverterOptions,
    poetry: &Poetry,
) -> Option<BuildBackendObject> {
    match &converter_options.build_backend {
        Some(BuildBackend::Hatch) | None => {
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
                        "Package distribution cound not be migrated to uv build backend due to the issues above. Consider using hatch build backend with \"{}\".",
                        "--build-backend hatch".bold(),
                    ));

                    None
                }
                Ok(None) => None,
            }
        }
    }
}
