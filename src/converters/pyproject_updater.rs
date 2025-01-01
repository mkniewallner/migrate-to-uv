use crate::schema::pep_621::Project;
use crate::schema::pyproject::{BuildSystem, DependencyGroupSpecification};
use crate::schema::uv::Uv;
use indexmap::IndexMap;
use toml_edit::{table, value, DocumentMut};

/// Updates a `pyproject.toml` document.
pub struct PyprojectUpdater<'a> {
    pub pyproject: &'a mut DocumentMut,
}

impl PyprojectUpdater<'_> {
    /// Adds or replaces PEP 621 data.
    pub fn insert_pep_621(&mut self, project: &Project) {
        self.pyproject["project"] = value(
            serde::Serialize::serialize(&project, toml_edit::ser::ValueSerializer::new()).unwrap(),
        );
    }

    /// Adds or replaces dependency groups data in TOML document.
    pub fn insert_dependency_groups(
        &mut self,
        dependency_groups: Option<&IndexMap<String, Vec<DependencyGroupSpecification>>>,
    ) {
        if let Some(dependency_groups) = dependency_groups {
            self.pyproject["dependency-groups"] = value(
                serde::Serialize::serialize(
                    &dependency_groups,
                    toml_edit::ser::ValueSerializer::new(),
                )
                .unwrap(),
            );
        }
    }

    /// Adds or replaces build system data.
    pub fn insert_build_system(&mut self, build_system: Option<&BuildSystem>) {
        if let Some(build_system) = build_system {
            self.pyproject["build-system"] = value(
                serde::Serialize::serialize(&build_system, toml_edit::ser::ValueSerializer::new())
                    .unwrap(),
            );
        }
    }

    /// Adds or replaces uv-specific data in TOML document.
    pub fn insert_uv(&mut self, uv: &Uv) {
        if uv == &Uv::default() {
            return;
        }

        if !self.pyproject.contains_key("tool") {
            self.pyproject["tool"] = table();
        }

        self.pyproject["tool"]["uv"] = value(
            serde::Serialize::serialize(&uv, toml_edit::ser::ValueSerializer::new()).unwrap(),
        );
    }
}
