use crate::schema::pipenv::Source;
use crate::schema::uv::Index;

pub fn get_indexes(pipenv_sources: Option<Vec<Source>>) -> Option<Vec<Index>> {
    Some(
        pipenv_sources?
            .iter()
            .map(|source| Index {
                name: source.name.to_string(),
                url: Some(source.url.to_string()),
                // https://pipenv.pypa.io/en/stable/indexes.html#index-restricted-packages
                explicit: if source.name.to_lowercase() == "pypi" {
                    None
                } else {
                    Some(true)
                },
                ..Default::default()
            })
            .collect(),
    )
}
