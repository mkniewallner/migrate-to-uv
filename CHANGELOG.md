# Changelog

## 0.7.1 - 2025-02-22

### Bug fixes

* Handle map for PEP 621 `license` field ([#156](https://github.com/mkniewallner/migrate-to-uv/pull/156))

## 0.7.0 - 2025-02-15

### Features

* Add `--skip-uv-checks` to skip checking if uv is already used in a project ([#118](https://github.com/mkniewallner/migrate-to-uv/pull/118))

### Bug fixes

* [pip/pip-tools] Warn on unhandled dependency formats ([#103](https://github.com/mkniewallner/migrate-to-uv/pull/103))
* [pip/pip-tools] Ignore inline comments when parsing dependencies ([#105](https://github.com/mkniewallner/migrate-to-uv/pull/105))
* [poetry] Migrate scripts that use `scripts = { callable = "foo:run" }` format instead of crashing ([#138](https://github.com/mkniewallner/migrate-to-uv/pull/138))

## 0.6.0 - 2025-01-20

Existing data in `[project]` section of `pyproject.toml` is now preserved by default when migrating. If you prefer that the section is fully replaced, this can be done by setting `--replace-project-section` flag, like so:

```bash
migrate-to-uv --replace-project-section
```

Poetry projects that use PEP 621 syntax to define project metadata, for which support was added in [Poetry 2.0](https://python-poetry.org/blog/announcing-poetry-2.0.0/), are now supported.

### Features

* Preserve existing data in `[project]` section of `pyproject.toml` when migrating ([#84](https://github.com/mkniewallner/migrate-to-uv/pull/84))
* [poetry] Support migrating projects using PEP 621 ([#85](https://github.com/mkniewallner/migrate-to-uv/pull/85))

## 0.5.0 - 2025-01-18

### Features

* [poetry] Delete `poetry.toml` after migration ([#62](https://github.com/mkniewallner/migrate-to-uv/pull/62))
* [pipenv] Delete `Pipfile.lock` after migration ([#66](https://github.com/mkniewallner/migrate-to-uv/pull/66))
* Exit if uv is detected as a package manager ([#61](https://github.com/mkniewallner/migrate-to-uv/pull/61))

### Bug fixes

* Ensure that lock file exists before parsing ([#67](https://github.com/mkniewallner/migrate-to-uv/pull/67))

### Documentation

* Explain how to set credentials for private indexes ([#60](https://github.com/mkniewallner/migrate-to-uv/pull/60))

## 0.4.0 - 2025-01-17

When generating `uv.lock` with `uv lock` command, `migrate-to-uv` now keeps the same versions dependencies were locked to with the previous package manager (if a lock file was found), both for direct and transitive dependencies. This is supported for Poetry, Pipenv, and pip-tools.

This new behavior can be opted out by setting `--ignore-locked-versions` flag, like so:

```bash
migrate-to-uv --ignore-locked-versions
```

### Features

* Keep locked dependencies versions when generating `uv.lock` ([#56](https://github.com/mkniewallner/migrate-to-uv/pull/56))

## 0.3.0 - 2025-01-12

Dependencies are now locked with `uv lock` at the end of the migration, if `uv` is detected as an executable. This new behavior can be opted out by setting `--skip-lock` flag, like so:

```bash
migrate-to-uv --skip-lock
```

### Features

* Lock dependencies at the end of migration ([#46](https://github.com/mkniewallner/migrate-to-uv/pull/46))

## 0.2.1 - 2025-01-05

### Bug fixes

* [poetry] Avoid crashing when an extra lists a non-existing dependency ([#30](https://github.com/mkniewallner/migrate-to-uv/pull/30))

## 0.2.0 - 2025-01-05

### Features

* Support migrating projects using `pip` and `pip-tools` ([#24](https://github.com/mkniewallner/migrate-to-uv/pull/24))
* [poetry] Migrate data from `packages`, `include` and `exclude` to Hatch build backend ([#16](https://github.com/mkniewallner/migrate-to-uv/pull/16))

## 0.1.2 - 2025-01-02

### Bug fixes

* [pipenv] Correctly update `pyproject.toml` ([#19](https://github.com/mkniewallner/migrate-to-uv/pull/19))
* Do not insert `[tool.uv]` if empty ([#17](https://github.com/mkniewallner/migrate-to-uv/pull/17))

## 0.1.1 - 2024-12-26

### Miscellaneous

* Fix documentation publishing and package metadata ([#3](https://github.com/mkniewallner/migrate-to-uv/pull/3))

## 0.1.0 - 2024-12-26

Initial release, with support for Poetry and Pipenv.
