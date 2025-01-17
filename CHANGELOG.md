# Changelog

## Unreleased

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
