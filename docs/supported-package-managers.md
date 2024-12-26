# Supported package managers

`migrate-to-uv` supports multiple package managers. By default, it tries to auto-detect the package manager based on the
files (and their content) used by the package managers it supports. If you need to enforce a specific package manager to
be used, use [`--package-manager`](usage-and-configuration.md#-package-manager).

## Poetry

Most [Poetry](https://python-poetry.org/) metadata is converted to uv when performing the migration:

- [Project metadata](https://python-poetry.org/docs/pyproject/) (`name`, `version`, `authors`, ...)
- [Dependencies and dependency groups](https://python-poetry.org/docs/pyproject/#dependencies-and-dependency-groups)
  (PyPI, path, git, URL)
- [Dependency extras](https://python-poetry.org/docs/pyproject/#extras) (also known as optional dependencies)
- [Dependency sources](https://python-poetry.org/docs/repositories/)
- [Dependency markers](https://python-poetry.org/docs/dependency-specification#using-environment-markers) (including
  [`python`](https://python-poetry.org/docs/dependency-specification/#python-restricted-dependencies) and `platform`)
- [Multiple constraints dependencies](https://python-poetry.org/docs/dependency-specification#multiple-constraints-dependencies)
- [Supported Python versions](https://python-poetry.org/docs/basic-usage/#setting-a-python-version)
- [Scripts](https://python-poetry.org/docs/pyproject/#scripts) and
  [plugins](https://python-poetry.org/docs/pyproject/#plugins) (also known as entry points)

Version definitions set for dependencies are also preserved, and converted to their
equivalent [PEP 440](https://peps.python.org/pep-0440/) format used by uv, even for Poetry-specific version
specification (e.g., [caret](https://python-poetry.org/docs/dependency-specification/#caret-requirements) (`^`)
and [tilde](https://python-poetry.org/docs/dependency-specification/#tilde-requirements) (`~`)).

### Missing features

The following features are not yet supported when migrating:

- Package distribution metadata ([`packages`](https://python-poetry.org/docs/pyproject/#packages), [`include` and `exclude`](https://python-poetry.org/docs/pyproject/#include-and-exclude))

## Pipenv

All existing [Pipenv](https://pipenv.pypa.io/en/stable/) metadata should be converted to uv when performing the
migration:

- [Dependencies and development dependencies](https://pipenv.pypa.io/en/stable/pipfile.html#example-pipfile) (PyPI,
  path, git, URL)
- [Package category groups](https://pipenv.pypa.io/en/stable/pipfile.html#package-category-groups)
- [Package indexes](https://pipenv.pypa.io/en/stable/indexes.html)
- [Dependency markers](https://pipenv.pypa.io/en/stable/specifiers.html#specifying-basically-anything)
- [Supported Python versions](https://pipenv.pypa.io/en/stable/advanced.html#automatic-python-installation)
