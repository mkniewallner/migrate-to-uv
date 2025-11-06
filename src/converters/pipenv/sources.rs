use crate::schema::pipenv::Source;
use crate::schema::uv::Index;

pub fn get_indexes(pipenv_sources: Option<Vec<Source>>) -> Option<Vec<Index>> {
    Some(
        pipenv_sources?
            .iter()
            .map(|source| Index {
                name: source.name.clone(),
                url: Some(source.url.clone()),
                // https://pipenv.pypa.io/en/stable/indexes.html#index-restricted-packages
                explicit: (source.name.to_lowercase() != "pypi").then_some(true),
                ..Default::default()
            })
            .collect(),
    )
}
