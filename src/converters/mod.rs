use crate::schema::pyproject::DependencyGroupSpecification;
use indexmap::IndexMap;

pub mod pip_tools;
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

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum DependencyGroupsStrategy {
    SetDefaultGroups,
    IncludeInDev,
    KeepExisting,
    MergeIntoDev,
}
