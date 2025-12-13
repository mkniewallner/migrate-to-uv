# Changelog

## 0.8.1 - 2025-12-13

### Bug fixes

* [poetry] Fail on unhandled version specifications ([#514](https://github.com/mkniewallner/migrate-to-uv/pull/514))
* [poetry] Also check if `poetry.lock` exists when checking if a project uses Poetry ([#528](https://github.com/mkniewallner/migrate-to-uv/pull/528))

## 0.8.0 - 2025-11-17

### Breaking changes

#### Abort on unrecoverable errors and warn about recoverable ones

Although `migrate-to-uv` tries its best to migrate a project to uv without changing the behaviour, some things that are accepted by package managers do not have any equivalent in uv. Previously, `migrate-to-uv` would warn about the issue, but still perform the migration. It now aborts the migration in case it is not able to perform the migration that would result in behaviour changes when migrating to uv.

If errors occur and lead to aborting the migration, you are expected to manually update your setup and retry the migration.

Warnings that occurred during the migration (which did not break the behaviour) are now also grouped and displayed at the very end of the migration.

### Features

* feat!: abort on unrecoverable errors and warn about recoverable ones ([#480](https://github.com/mkniewallner/migrate-to-uv/pull/480))
* [poetry] Do not set optional groups as default ones ([#299](https://github.com/mkniewallner/migrate-to-uv/pull/299))
* Indicate support for Python 3.14 ([#468](https://github.com/mkniewallner/migrate-to-uv/pull/468))

### Bug fixes

* [poetry] Use inclusion when converting `^x.y` versions ([#466](https://github.com/mkniewallner/migrate-to-uv/pull/466))
* [poetry] Properly convert `include` to Hatch's build backend ([#477](https://github.com/mkniewallner/migrate-to-uv/pull/477))
* [poetry] Do not crash on empty `readme` array ([#481](https://github.com/mkniewallner/migrate-to-uv/pull/481))
* [poetry] Abort migration on dependencies using `||` operator, as there is no PEP 440 equivalent ([#487](https://github.com/mkniewallner/migrate-to-uv/pull/487))
* [poetry] Handle versions that use Poetry style and `,`, like `^1.0,!=1.1.0` ([#489](https://github.com/mkniewallner/migrate-to-uv/pull/489))
* [pip/pip-tools] Suggest how to add dependencies that could not be converted ([#350](https://github.com/mkniewallner/migrate-to-uv/pull/350))
* Preserve comments for sections unrelated to migration ([#471](https://github.com/mkniewallner/migrate-to-uv/pull/471))

## 0.7.3 - 2025-06-07

### Bug fixes

* Use correct `include-group` name to include groups when using `--dependency-groups-strategy include-in-dev` ([#283](https://github.com/mkniewallner/migrate-to-uv/pull/283))
* [poetry] Handle `=` single equality ([#288](https://github.com/mkniewallner/migrate-to-uv/pull/288))
* [pipenv] Handle raw versions ([#292](https://github.com/mkniewallner/migrate-to-uv/pull/292))

## 0.7.2 - 2025-03-25

### Bug fixes

* [pipenv] Handle `*` for version ([#212](https://github.com/mkniewallner/migrate-to-uv/pull/212))

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
