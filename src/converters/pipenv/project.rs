use crate::schema::pipenv::Requires;

pub fn get_requires_python(pipenv_requires: Option<Requires>) -> Option<String> {
    let pipenv_requires = pipenv_requires?;

    if let Some(python_version) = pipenv_requires.python_version {
        return Some(format!("~={python_version}"));
    }

    if let Some(python_full_version) = pipenv_requires.python_full_version {
        return Some(format!("=={python_full_version}"));
    }

    None
}
