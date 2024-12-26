`migrate-to-uv` migrates a project to [uv](https://github.com/astral-sh/uv) from another package manager.

Try it now:

```bash
# With uv
uvx migrate-to-uv

# With pipx
pipx run migrate-to-uv
```

The following package managers are supported:

- [Poetry](supported-package-managers.md#poetry)
- [Pipenv](supported-package-managers.md#pipenv)

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
[caret](https://python-poetry.org/docs/dependency-specification/#caret-requirements) for Poetry).

!!! warning

    Although `migrate-to-uv` matches current package manager definition as closely as possible when performing the migration, it is still heavily recommended to double check the end result, especially if you are migrating a package that is meant to be publicly distributed.
    
    If you notice a behaviour that does not match the previous package manager when migrating, please [raise an issue](https://github.com/mkniewallner/migrate-to-uv/issues), if not already reported.
