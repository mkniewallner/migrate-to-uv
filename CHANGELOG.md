# Changelog

## 0.1.2 - 2025-01-02

### Features

* [poetry] Migrate data from `packages`, `include` and `exclude` to hatch build backend ([#16](https://github.com/mkniewallner/migrate-to-uv/pull/16))

### Bug fixes

* [pipenv] Correctly update `pyproject.toml` ([#19](https://github.com/mkniewallner/migrate-to-uv/pull/19))
* Do not insert `[tool.uv]` if empty ([#17](https://github.com/mkniewallner/migrate-to-uv/pull/17))

## 0.1.1 - 2024-12-26

### Miscellaneous

* Fix documentation publishing and package metadata ([#3](https://github.com/mkniewallner/migrate-to-uv/pull/3))

## 0.1.0 - 2024-12-26

Initial release, with support for Poetry and Pipenv.
