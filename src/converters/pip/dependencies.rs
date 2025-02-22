use log::warn;
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
            let line = line.trim();

            // Ignore empty lines, comments. Also ignore lines starting with `-` to ignore arguments
            // (package names cannot start with a hyphen). No argument is supported yet, so we can
            // simply ignore all of them.
            if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
                continue;
            }

            let dependency = match line.split_once(" #") {
                Some((dependency, _)) => dependency,
                None => line,
            };

            let dependency_specification = Requirement::<Url>::from_str(dependency);

            if let Ok(dependency_specification) = dependency_specification {
                dependencies.push(dependency_specification.to_string());
            } else {
                warn!(
                    "Could not parse the following dependency specification as a PEP 508 one: {line}"
                );
            }
        }
    }

    if dependencies.is_empty() {
        return None;
    }
    Some(dependencies)
}
