use crate::schema::pep_621::AuthorOrMaintainer;
use crate::schema::poetry::Script;
use crate::schema::utils::SingleOrVec;
use indexmap::IndexMap;
use log::warn;
use regex::Regex;
use std::sync::LazyLock;

static AUTHOR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?<name>[^<>]+)(?: <(?<email>.+?)>)?$").unwrap());

pub fn get_readme(poetry_readme: Option<SingleOrVec<String>>) -> Option<String> {
    poetry_readme.map(|readme| match readme {
        SingleOrVec::Single(readme) => readme,
        SingleOrVec::Vec(readmes) => {
            warn!("Found multiple readme files ({}). PEP 621 only supports setting one, so only the first one was added.", readmes.join(", "));
            readmes[0].clone()
        }
    })
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
