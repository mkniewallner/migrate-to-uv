use crate::schema::poetry::{IncludeExclude, Package};
use crate::schema::pyproject::BuildSystem;
use log::warn;
use owo_colors::OwoColorize;

pub fn get_new_build_system(build_system: Option<BuildSystem>) -> Option<BuildSystem> {
    if build_system?.build_backend? == "poetry.core.masonry.api" {
        return Some(BuildSystem {
            requires: vec!["uv>=0.5,<0.6".to_string()],
            build_backend: Some("uv".to_string()),
        });
    }
    None
}

/// Warns that migration of package-related keys is not yet supported, if we find them.
pub fn warn_unsupported_package_keys(
    packages: Option<&Vec<Package>>,
    include: Option<&Vec<IncludeExclude>>,
    exclude: Option<&Vec<IncludeExclude>>,
) {
    let mut detected_package_keys = Vec::new();

    if packages.is_some() {
        detected_package_keys.push("packages");
    }
    if include.is_some() {
        detected_package_keys.push("include");
    }
    if exclude.is_some() {
        detected_package_keys.push("exclude");
    }

    if !detected_package_keys.is_empty() {
        warn!("Migration of package specification is not yet supported, so the following keys under {} were not migrated: {}.", "[tool.poetry]".bold(), detected_package_keys.join(", ").bold());
    }
}
