`migrate-to-uv` migrates a project to [uv](https://github.com/astral-sh/uv) from another package manager.

Try it now:

```bash
# With uv
uvx migrate-to-uv

# With pipx
pipx run migrate-to-uv
```

The following package managers are supported:

- [Poetry](supported-package-managers.md#poetry) (including projects
  using [PEP 621 in Poetry 2.0+](https://python-poetry.org/blog/announcing-poetry-2.0.0/))
- [Pipenv](supported-package-managers.md#pipenv)
- [pip-tools](supported-package-managers.md#pip-tools)
- [pip](supported-package-managers.md#pip)

More package managers (e.g., [setuptools](https://setuptools.pypa.io/en/stable/)) could be implemented in the
future.

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

At the end of the migration, `migrate-to-uv` also generates `uv.lock` file with `uv lock` command to lock dependencies,
and keeps dependencies (both direct and transitive) to the exact same versions they were locked to with the previous
package manager, if a lock file was found.

!!! warning

    Although `migrate-to-uv` matches current package manager definition as closely as possible when performing the migration, it is still heavily recommended to double check the end result, especially if you are migrating a package that is meant to be publicly distributed.
    
    If you notice a behaviour that does not match the previous package manager when migrating, please [raise an issue](https://github.com/mkniewallner/migrate-to-uv/issues), if not already reported.
