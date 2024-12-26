use crate::detector::PackageManager;
use crate::schema::pyproject::DependencyGroupSpecification;
use indexmap::IndexMap;
use std::path::PathBuf;

pub mod pipenv;
pub mod poetry;
mod pyproject_updater;

type DependencyGroupsAndDefaultGroups = (
    Option<IndexMap<String, Vec<DependencyGroupSpecification>>>,
    Option<Vec<String>>,
);

/// Converts a project from a package manager to uv.
pub trait Converter {
    fn convert_to_uv(
        &self,
        dry_run: bool,
        keep_old_metadata: bool,
        dependency_groups_strategy: DependencyGroupsStrategy,
    );
}

pub fn get_converter(
    detected_package_manager: &PackageManager,
    project_path: PathBuf,
) -> Box<dyn Converter> {
    match detected_package_manager {
        PackageManager::Pipenv => Box::new(pipenv::Pipenv { project_path }),
        PackageManager::Poetry => Box::new(poetry::Poetry { project_path }),
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum DependencyGroupsStrategy {
    SetDefaultGroups,
    IncludeInDev,
    KeepExisting,
    MergeIntoDev,
}
