use crate::schema::pep_621::AuthorOrMaintainer;
use configparser::ini::Ini;
use indexmap::IndexMap;
use log::warn;

pub fn get_version(config: &Ini) -> String {
    match config.get("metadata", "version") {
        Some(version) if version.starts_with("attr:") || version.starts_with("file:") => {
            warn!("\"version\" uses a dynamic attribute, which is not supported.");
            "0.0.1".to_string()
        }
        Some(version) => version,
        None => "0.0.1".to_string(),
    }
}

pub fn get_authors(name: Option<String>, email: Option<String>) -> Option<Vec<AuthorOrMaintainer>> {
    if name.is_none() && email.is_none() {
        return None;
    }
    Some(vec![AuthorOrMaintainer { name, email }])
}

pub fn get_urls(_config: Ini) -> Option<IndexMap<String, String>> {
    let urls: IndexMap<String, String> = IndexMap::new();

    if urls.is_empty() {
        return None;
    }

    Some(urls)
}
