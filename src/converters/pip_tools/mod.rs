use crate::converters::pyproject_updater::PyprojectUpdater;
use crate::converters::Converter;
use crate::converters::DependencyGroupsStrategy;
use crate::schema::pep_621::Project;
use crate::schema::uv::Uv;
use crate::toml::PyprojectPrettyFormatter;
use log::info;
use owo_colors::OwoColorize;
use std::default::Default;
use std::fs;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use toml_edit::visit_mut::VisitMut;
use toml_edit::DocumentMut;

pub struct PipTools {
    pub project_path: PathBuf,
    pub requirements_files: Vec<String>,
    pub dev_requirements_files: Vec<String>,
}

impl Converter for PipTools {
    fn convert_to_uv(
        &self,
        dry_run: bool,
        keep_old_metadata: bool,
        dependency_groups_strategy: DependencyGroupsStrategy,
    ) {
        let pyproject_path = self.project_path.join("pyproject.toml");
        let updated_pyproject_string = perform_migration(
            &self.project_path,
            self.requirements_files.clone(),
            self.dev_requirements_files.clone(),
            &pyproject_path,
            dependency_groups_strategy,
        );

        if dry_run {
            info!(
                "{}\n{}",
                "Migrated pyproject.toml:".bold(),
                updated_pyproject_string
            );
        } else {
            let mut pyproject_file = File::create(&pyproject_path).unwrap();

            pyproject_file
                .write_all(updated_pyproject_string.as_bytes())
                .unwrap();

            if !keep_old_metadata {
                delete_pip_tools_references(&self.project_path).unwrap();
            }

            info!(
                "{}",
                "Successfully migrated project from pip-tools to uv!\n"
                    .bold()
                    .green()
            );
        }
    }
}

fn perform_migration(
    project_path: &Path,
    requirements_files: Vec<String>,
    dev_requirements_files: Vec<String>,
    pyproject_path: &Path,
    _dependency_groups_strategy: DependencyGroupsStrategy,
) -> String {
    for requirements_file in requirements_files {
        let requirements_content =
            fs::read_to_string(project_path.join(requirements_file)).unwrap();

        for line in requirements_content.lines() {
            println!("{line}");
        }
    }

    for dev_requirements_file in dev_requirements_files {
        let requirements_content =
            fs::read_to_string(project_path.join(dev_requirements_file)).unwrap();

        for line in requirements_content.lines() {
            println!("{line}");
        }
    }

    let project = Project {
        // "name" is required by uv.
        name: Some(String::new()),
        // "version" is required by uv.
        ..Default::default()
    };

    let uv = Uv {
        package: Some(false),
        ..Default::default()
    };

    let pyproject_toml_content = fs::read_to_string(pyproject_path).unwrap_or_default();
    let mut updated_pyproject = pyproject_toml_content.parse::<DocumentMut>().unwrap();
    let mut pyproject_updater = PyprojectUpdater {
        pyproject: &mut updated_pyproject,
    };

    pyproject_updater.insert_pep_621(&project);
    pyproject_updater.insert_uv(&uv);

    let mut visitor = PyprojectPrettyFormatter {
        parent_keys: Vec::new(),
    };
    visitor.visit_document_mut(&mut updated_pyproject);

    updated_pyproject.to_string()
}

fn delete_pip_tools_references(project_path: &Path) -> std::io::Result<()> {
    let requirements_path = project_path.join("requirements.in");

    if requirements_path.exists() {
        remove_file(requirements_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perform_migration() {
        insta::assert_toml_snapshot!(perform_migration(
            Path::new("tests/fixtures/pip_tools/full"),
            vec!["requirements.in".to_string()],
            vec!["requirements-dev.in".to_string()],
            Path::new("tests/fixtures/pip_tools/full/pyproject.toml"),
            DependencyGroupsStrategy::SetDefaultGroups,
        ));
    }
}
