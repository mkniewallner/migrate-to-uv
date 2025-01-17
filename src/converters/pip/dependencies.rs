use pep508_rs::Requirement;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use url::Url;

pub fn get(project_path: &Path, requirements_files: Vec<String>) -> Option<Vec<String>> {
    let mut dependencies: Vec<String> = Vec::new();

    for requirements_file in requirements_files {
        let requirements_content =
            fs::read_to_string(project_path.join(requirements_file)).unwrap();

        for line in requirements_content.lines() {
            let dependency_specification = Requirement::<Url>::from_str(line);

            if let Ok(dependency_specification) = dependency_specification {
                dependencies.push(dependency_specification.to_string());
            }
        }
    }

    if dependencies.is_empty() {
        return None;
    }
    Some(dependencies)
}

pub fn get_constraint_dependencies(
    ignore_locked_versions: bool,
    project_path: &Path,
    requirements_files: Vec<String>,
    dev_requirements_files: Vec<String>,
) -> Option<Vec<String>> {
    if ignore_locked_versions {
        return None;
    }

    if let Some(dependencies) = get(
        project_path,
        requirements_files
            .into_iter()
            .chain(dev_requirements_files)
            .map(|f| f.replace(".in", ".txt"))
            .collect(),
    ) {
        if dependencies.is_empty() {
            return None;
        }
        return Some(dependencies);
    }
    None
}
