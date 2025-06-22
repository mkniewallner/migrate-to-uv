use log::warn;
use pep508_rs::Requirement;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use url::Url;

pub fn get(project_path: &Path, requirements_files: &[String]) -> Option<Vec<String>> {
    let mut dependencies: Vec<String> = Vec::new();

    for requirements_file in requirements_files {
        let requirements_content =
            fs::read_to_string(project_path.join(requirements_file)).unwrap();

        for line in requirements_content.lines() {
            let line = line.trim();

            // Ignore empty lines and comments.
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // https://pip.pypa.io/en/stable/reference/requirements-file-format/#referring-to-other-requirements-files
            // For `-r`, pip allows both `-r requirements.txt` and `-rrequirements.txt`.
            // For `--requirement`, pip only allows `--requirement requirements.txt`.
            // For both options, an infinite number of spaces is allowed between the argument and
            // its value.
            if line.starts_with("-r") || line.starts_with("--requirement ") {
                let prefix = if line.starts_with("-r") {
                    "-r"
                } else {
                    "--requirement"
                };

                let nested_requirements_file = line.strip_prefix(prefix).unwrap_or_default().trim();

                // If references requirements file is already passed as an argument, skip it, to not
                // add dependencies twice.
                if requirements_files.contains(&nested_requirements_file.to_string()) {
                    continue;
                }

                if project_path.join(nested_requirements_file).exists() {
                    dependencies.extend(
                        get(project_path, &[nested_requirements_file.to_string()])
                            .unwrap_or_default(),
                    );
                } else {
                    warn!(
                        "Could not resolve \"{nested_requirements_file}\" referenced in \"{requirements_file}\"."
                    );
                }

                continue;
            }

            // Ignore lines starting with `-` to ignore other arguments (package names cannot start
            // with a hyphen), as besides `-r`, they are unsupported.
            if line.starts_with('-') {
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
