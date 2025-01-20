# Supported package managers

`migrate-to-uv` supports multiple package managers. By default, it tries to auto-detect the package manager based on the
files (and their content) used by the package managers it supports. If you need to enforce a specific package manager to
be used, use [`--package-manager`](usage-and-configuration.md#-package-manager).

## Poetry

!!! note

    `migrate-to-uv` supports migrating both projects that use Poetry-specific syntax for defining project metadata, and
    projects that use PEP 621, added in [Poetry 2.0](https://python-poetry.org/blog/announcing-poetry-2.0.0/).

All existing [Poetry](https://python-poetry.org/) metadata should be converted to uv when performing the migration:

- [Project metadata](https://python-poetry.org/docs/pyproject/) (`name`, `version`, `authors`, ...)
- [Dependencies and dependency groups](https://python-poetry.org/docs/pyproject/#dependencies-and-dependency-groups)
  (PyPI, path, git, URL)
- [Dependency extras](https://python-poetry.org/docs/pyproject/#extras) (also known as optional dependencies)
- [Dependency sources](https://python-poetry.org/docs/repositories/)
- [Dependency markers](https://python-poetry.org/docs/dependency-specification/#using-environment-markers) (including
  [`python`](https://python-poetry.org/docs/dependency-specification/#python-restricted-dependencies) and `platform`)
- [Multiple constraints dependencies](https://python-poetry.org/docs/dependency-specification/#multiple-constraints-dependencies)
- Package distribution metadata ([`packages`](https://python-poetry.org/docs/pyproject/#packages), [`include` and `exclude`](https://python-poetry.org/docs/pyproject/#exclude-and-include))
- [Supported Python versions](https://python-poetry.org/docs/basic-usage/#setting-a-python-version)
- [Scripts](https://python-poetry.org/docs/pyproject/#scripts) and
  [plugins](https://python-poetry.org/docs/pyproject/#plugins) (also known as entry points)

Version definitions set for dependencies are also preserved, and converted to their
equivalent [PEP 440](https://peps.python.org/pep-0440/) format used by uv, even for Poetry-specific version
specification (e.g., [caret](https://python-poetry.org/docs/dependency-specification/#caret-requirements) (`^`)
and [tilde](https://python-poetry.org/docs/dependency-specification/#tilde-requirements) (`~`)).

### Build backend

As uv does not yet have a stable build backend (see [astral-sh/uv#8779](https://github.com/astral-sh/uv/issues/8779) for more details), when
performing the migration for libraries, `migrate-to-uv` sets [Hatch](https://hatch.pypa.io/latest/) as a build
backend, migrating:

- Poetry [`packages`](https://python-poetry.org/docs/pyproject/#packages) and [`include`](https://python-poetry.org/docs/pyproject/#exclude-and-include) to Hatch [`include`](https://hatch.pypa.io/latest/config/build/#patterns)
- Poetry [`exclude`](https://python-poetry.org/docs/pyproject/#exclude-and-include) to Hatch [`exclude`](https://hatch.pypa.io/latest/config/build/#patterns)

!!! note

    Path rewriting, defined with `to` in `packages` for Poetry, is also migrated to Hatch by defining
    [sources](https://hatch.pypa.io/latest/config/build/#rewriting-paths) in wheel target.


Once uv build backend is out of preview and considered stable, it will be used for the migration.

## Pipenv

All existing [Pipenv](https://pipenv.pypa.io/en/stable/) metadata should be converted to uv when performing the
migration:

- [Dependencies and development dependencies](https://pipenv.pypa.io/en/stable/pipfile.html#example-pipfile) (PyPI,
  path, git, URL)
- [Package category groups](https://pipenv.pypa.io/en/stable/pipfile.html#package-category-groups)
- [Package indexes](https://pipenv.pypa.io/en/stable/indexes.html)
- [Dependency markers](https://pipenv.pypa.io/en/stable/specifiers.html#specifying-basically-anything)
- [Supported Python versions](https://pipenv.pypa.io/en/stable/advanced.html#automatic-python-installation)

## pip-tools

Most [pip-tools](https://pip-tools.readthedocs.io/en/stable/) metadata is converted to uv when performing the migration.

By default, `migrate-to-uv` will search for:

- production dependencies in `requirements.in`
- development dependencies in `requirements-dev.in`

If your project uses different file names, or defines production and/or development dependencies across multiple files,
you can specify the names of the files using [`--requirements-file`](usage-and-configuration.md#-requirements-file) and
[`--dev-requirements-file`](usage-and-configuration.md#-dev-requirements-file) (both can be specified multiple times),
for instance:

```bash
migrate-to-uv \
  --requirements-file requirements-prod.in \
  --dev-requirements-file requirements-dev.in \
  --dev-requirements-file requirements-docs.in
```

### Missing features

- Dependencies that do not follow [PEP 508](https://peps.python.org/pep-0508/) specification are not yet handled
- References to other requirement files (e.g., `-r other-requirements.in`) are not supported, but the requirements file
  can manually be set with [`--requirements-file`](usage-and-configuration.md#-requirements-file) or
  [`--dev-requirements-file`](usage-and-configuration.md#-dev-requirements-file)
- Index URLs are not yet migrated

## pip

Most [pip](https://pip.pypa.io/en/stable/) metadata is converted to uv when performing the migration.

By default, `migrate-to-uv` will search for:

- production dependencies in `requirements.txt`
- development dependencies in `requirements-dev.txt`

If your project uses different file names, or defines production and/or development dependencies across multiple files,
you can specify the names of the files [`--requirements-file`](usage-and-configuration.md#-requirements-file) and
[`--dev-requirements-file`](usage-and-configuration.md#-dev-requirements-file) (both can be specified multiple times),
for instance:

```bash
migrate-to-uv \
  --requirements-file requirements-prod.txt \
  --dev-requirements-file requirements-dev.txt \
  --dev-requirements-file requirements-docs.txt
```

### Missing features

- Dependencies that do not follow [PEP 508](https://peps.python.org/pep-0508/) specification are not yet handled
- References to other requirement files (e.g., `-r other-requirements.txt`) are not supported, but the requirements file
  can manually be set with [`--requirements-file`](usage-and-configuration.md#-requirements-file) or
  [`--dev-requirements-file`](usage-and-configuration.md#-dev-requirements-file)
- Index URLs are not yet migrated
