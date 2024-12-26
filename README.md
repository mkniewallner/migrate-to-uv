# migrate-to-uv

[![PyPI](https://img.shields.io/pypi/v/migrate-to-uv.svg)](https://pypi.org/project/migrate-to-uv/)
[![License](https://img.shields.io/pypi/l/migrate-to-uv.svg)](https://pypi.org/project/migrate-to-uv/)
[![Supported Python versions](https://img.shields.io/pypi/pyversions/migrate-to-uv.svg)](https://pypi.org/project/migrate-to-uv/)

`migrate-to-uv` migrates a project to [uv](https://github.com/astral-sh/uv) from another package manager.

## Usage

```bash
# With uv
uvx migrate-to-uv

# With pipx
pipx run migrate-to-uv
```

## Supported package managers

The following package managers are supported:

- [Poetry](https://python-poetry.org/)
- [Pipenv](https://pipenv.pypa.io/en/stable/)

More package managers (e.g., [pip-tools](https://pip-tools.readthedocs.io/en/stable/),
[pip](https://pip.pypa.io/en/stable/), [setuptools](https://setuptools.pypa.io/en/stable/), ...) could be
implemented in the future.

## Features

`migrate-to-uv` converts most existing metadata from supported package managers when migrating to uv, including:

- [Project metadata](https://packaging.python.org/en/latest/guides/writing-pyproject-toml/#writing-pyproject-toml) (`name`, `version`, `authors`, ...)
- [Dependencies and optional dependencies](https://packaging.python.org/en/latest/guides/writing-pyproject-toml/#dependencies-optional-dependencies)
- [Dependency groups](https://peps.python.org/pep-0735/)
- [Dependency sources](https://docs.astral.sh/uv/concepts/projects/dependencies/#dependency-sources) (index, git, URL, path)
- [Dependency markers](https://packaging.python.org/en/latest/specifications/dependency-specifiers/)
- [Entry points](https://packaging.python.org/en/latest/specifications/pyproject-toml/#entry-points)

Version definitions set for dependencies are also preserved, and converted to their
equivalent [PEP 440](https://peps.python.org/pep-0440/) for package managers that use their own syntax (for instance
Poetry's [caret](https://python-poetry.org/docs/dependency-specification/#caret-requirements) syntax).
