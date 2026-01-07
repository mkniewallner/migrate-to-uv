use crate::errors::add_unrecoverable_error;
use crate::schema::pep_621::{AuthorOrMaintainer, Project};
use crate::schema::poetry::Script;
use crate::schema::pyproject::BuildSystem;
use crate::schema::utils::SingleOrVec;
use indexmap::{IndexMap, IndexSet};
use owo_colors::OwoColorize;
use pep440_rs::{Version, VersionSpecifiers};
use regex::Regex;
use std::str::FromStr;
use std::sync::LazyLock;

const EARLIEST_SUPPORTED_PYTHON_3_MINOR_VERSION: u8 = 4;
const LATEST_SUPPORTED_PYTHON_3_MINOR_VERSION: u8 = 14;
const PYTHON_CLASSIFIER_PREFIX: &str = "Programming Language :: Python";

static AUTHOR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?<name>[^<>]+)(?: <(?<email>.+?)>)?$").unwrap());

pub fn get_readme(poetry_readme: Option<SingleOrVec<String>>) -> Option<String> {
    match poetry_readme {
        Some(SingleOrVec::Single(readme)) => Some(readme),
        Some(SingleOrVec::Vec(readmes)) => match readmes.as_slice() {
            [] => None,
            [readme] => Some(readme.clone()),
            _ => {
                add_unrecoverable_error(format!(
                    "Found multiple files ({}) in \"{}\". PEP 621 only supports setting one. Make sure to manually edit the section before migrating.",
                    readmes
                        .iter()
                        .map(|r| format!("\"{}\"", r.bold()))
                        .collect::<Vec<String>>()
                        .join(", "),
                    "tool.poetry.readme".bold(),
                ));
                None
            }
        },
        None => None,
    }
}

pub fn get_authors(authors: Option<Vec<String>>) -> Option<Vec<AuthorOrMaintainer>> {
    Some(
        authors?
            .iter()
            .map(|p| {
                let captures = AUTHOR_REGEX.captures(p).unwrap();

                AuthorOrMaintainer {
                    name: captures.name("name").map(|m| m.as_str().into()),
                    email: captures.name("email").map(|m| m.as_str().into()),
                }
            })
            .collect(),
    )
}

pub fn get_urls(
    poetry_urls: Option<IndexMap<String, String>>,
    homepage: Option<String>,
    repository: Option<String>,
    documentation: Option<String>,
) -> Option<IndexMap<String, String>> {
    let mut urls: IndexMap<String, String> = IndexMap::new();

    if let Some(homepage) = homepage {
        urls.insert("Homepage".to_string(), homepage);
    }

    if let Some(repository) = repository {
        urls.insert("Repository".to_string(), repository);
    }

    if let Some(documentation) = documentation {
        urls.insert("Documentation".to_string(), documentation);
    }

    // URLs defined under `[tool.poetry.urls]` override whatever is set in `repository` or
    // `documentation` if there is a case-sensitive match. This is not the case for `homepage`, but
    // this is probably not an edge case worth handling.
    if let Some(poetry_urls) = poetry_urls {
        urls.extend(poetry_urls);
    }

    if urls.is_empty() {
        return None;
    }

    Some(urls)
}

pub fn get_scripts(
    poetry_scripts: Option<IndexMap<String, Script>>,
    scripts_from_plugins: Option<IndexMap<String, String>>,
) -> Option<IndexMap<String, String>> {
    let mut scripts: IndexMap<String, String> = IndexMap::new();

    if let Some(poetry_scripts) = poetry_scripts {
        for (name, script) in poetry_scripts {
            match script {
                Script::String(script) => {
                    scripts.insert(name, script);
                }
                Script::Map { callable } => {
                    if let Some(callable) = callable {
                        scripts.insert(name, callable);
                    }
                }
            }
        }
    }

    if let Some(scripts_from_plugins) = scripts_from_plugins {
        scripts.extend(scripts_from_plugins);
    }

    if scripts.is_empty() {
        return None;
    }
    Some(scripts)
}

/// Build classifiers to set under `classifiers`.
///
/// It includes Python supported version ones which are automatically by Poetry
/// (<https://python-poetry.org/docs/pyproject#classifiers-1>) when building distributions based on
/// `python` specifier and a hardcoded list of Python majors and minors
/// (<https://github.com/python-poetry/poetry-core/blob/2.2.1/src/poetry/core/packages/package.py#L40-L55>).
///
/// Note that for Poetry projects using PEP 621 and setting a `classifiers` key, Python classifiers
/// are not automatically added (<https://python-poetry.org/docs/pyproject#classifiers>).
pub fn get_classifiers(
    classifiers: Option<Vec<String>>,
    build_system: Option<&BuildSystem>,
    requires_python: Option<String>,
    project: Option<&Project>,
) -> Option<Vec<String>> {
    // Using an IndexSet ensures that we keep the previous order, while also remove duplicate, in
    // case for instance Python classifiers are also manually set.
    let mut classifiers: IndexSet<String> = IndexSet::from_iter(classifiers.unwrap_or_default());

    let has_pep_621_classifiers = project.is_some_and(|p| p.classifiers.is_some());

    // If we did not find Poetry build system, we're likely not migrating a package, or the package
    // does not use Poetry as a build backend, so we don't want to add classifiers. We completely
    // skip this if we found a PEP 621 classifiers, as in that case Poetry does not automatically
    // add classifiers.
    if build_system.is_some() && !has_pep_621_classifiers {
        // Python specification can come from either `python` under `[tool.poetry.dependencies]` or
        // `requires-python` for Poetry projects using PEP 621.
        let python_spec =
            requires_python.or_else(|| project.and_then(|p| p.requires_python.clone()));

        // If we found a Python specifier, and we are able to parse it as a PEP 508 specifier, we
        // assert if each possible classifier handled by Poetry is contained in the specifier, and
        // add the classifier if that's the case.
        if let Some(python_spec) = python_spec
            && let Ok(python_spec) = VersionSpecifiers::from_str(python_spec.as_str())
        {
            if python_spec.contains(&Version::from_str("2.7").unwrap()) {
                classifiers.insert(format!("{PYTHON_CLASSIFIER_PREFIX} :: 2"));
                classifiers.insert(format!("{PYTHON_CLASSIFIER_PREFIX} :: 2.7"));
            }

            for version in
                EARLIEST_SUPPORTED_PYTHON_3_MINOR_VERSION..=LATEST_SUPPORTED_PYTHON_3_MINOR_VERSION
            {
                if python_spec
                    .contains(&Version::from_str(format!("3.{version}").as_str()).unwrap())
                {
                    classifiers.insert(format!("{PYTHON_CLASSIFIER_PREFIX} :: 3"));
                    classifiers.insert(format!("{PYTHON_CLASSIFIER_PREFIX} :: 3.{version}"));
                }
            }
        // When no Python specifier is set, Poetry adds:
        // - Python 2 and 2.7 specifiers
        // - Python 3 and 3.x specifiers, where x ranges from 4 to the latest supported version that
        //   is manually updated over the years.
        } else {
            classifiers.insert(format!("{PYTHON_CLASSIFIER_PREFIX} :: 2"));
            classifiers.insert(format!("{PYTHON_CLASSIFIER_PREFIX} :: 2.7"));
            classifiers.insert(format!("{PYTHON_CLASSIFIER_PREFIX} :: 3"));

            for version in
                EARLIEST_SUPPORTED_PYTHON_3_MINOR_VERSION..=LATEST_SUPPORTED_PYTHON_3_MINOR_VERSION
            {
                classifiers.insert(format!("{PYTHON_CLASSIFIER_PREFIX} :: 3.{version}"));
            }
        }
    }

    if classifiers.is_empty() {
        return None;
    }
    Some(classifiers.into_iter().collect())
}
