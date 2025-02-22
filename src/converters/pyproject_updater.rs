use crate::schema::hatch::Hatch;
use crate::schema::pep_621::Project;
use crate::schema::pyproject::{BuildSystem, DependencyGroupSpecification};
use crate::schema::uv::Uv;
use indexmap::IndexMap;
use toml_edit::{DocumentMut, table, value};

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

    /// Adds or replaces hatch-specific data in TOML document.
    pub fn insert_hatch(&mut self, hatch: Option<&Hatch>) {
        if hatch.is_none() {
            return;
        }

        if !self.pyproject.contains_key("tool") {
            self.pyproject["tool"] = table();
        }

        self.pyproject["tool"]["hatch"] = value(
            serde::Serialize::serialize(&hatch, toml_edit::ser::ValueSerializer::new()).unwrap(),
        );
    }

    /// Remove `constraint-dependencies` under `[tool.uv]`, which is only needed to lock
    /// dependencies to specific versions in the generated lock file.
    pub fn remove_constraint_dependencies(&mut self) -> Option<&DocumentMut> {
        self.pyproject
            .get_mut("tool")?
            .as_table_mut()?
            .get_mut("uv")?
            .as_table_mut()?
            .remove("constraint-dependencies")?;

        // If `constraint-dependencies` was the only item in `[tool.uv]`, remove `[tool.uv]`.
        if self
            .pyproject
            .get("tool")?
            .as_table()?
            .get("uv")?
            .as_table()?
            .is_empty()
        {
            self.pyproject
                .get_mut("tool")?
                .as_table_mut()?
                .remove("uv")?;
        }

        Some(self.pyproject)
    }
}
