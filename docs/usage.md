---
icon: lucide/play
---
# Usage

## Basic usage

```bash
# With uv
uvx migrate-to-uv

# With pipx
pipx run migrate-to-uv
```

For more advanced usages, see the [configuration](configuration.md) options.

## Migration errors

Although `migrate-to-uv` tries its best to match the current package manager definition when performing the migration,
some package managers have features that have no equivalent in uv or
in [PEP 621](https://packaging.python.org/en/latest/specifications/pyproject-toml/#pyproject-toml-spec) specification
that is followed by uv.

In case the current package manager definition uses features that cannot be translated to uv, `migrate-to-uv` will abort
the migration, pointing at the errors, and suggesting what to do before attempting the migration again, e.g.:

```console
$ uvx migrate-to-uv
error: Could not automatically migrate the project to uv because of the following errors:
error: - Found multiple files ("README.md", "README2.md") in "tool.poetry.readme". PEP 621 only supports setting one. Make sure to manually edit the section before migrating.
```

For less problematic issues, `migrate-to-uv` will still perform the migration, but warn about what needs attention at
the end of it, e.g.:

```console
$ uvx migrate-to-uv
[...]
Successfully migrated project from Poetry to uv!

warning: The following warnings occurred during the migration:
warning: - Could not find dependency "non-existing-dependency" listed in "extra-with-non-existing-dependencies" extra.
```

## Authentication for private indexes

By default, `migrate-to-uv` generates `uv.lock` with `uv lock` to lock dependencies. If you currently use a package
manager with private indexes, credentials will need to be set for locking to work properly. This can be done by setting
the [same environment variables as uv expects for private indexes](https://docs.astral.sh/uv/concepts/indexes/#providing-credentials-directly).

Since the names of the indexes in uv should be the same as the ones in the current package manager before the migration,
you should be able to adapt the environment variables based on what you previously used.

For instance, if you currently use Poetry and have:

```toml
[[tool.poetry.source]]
name = "foo-bar"
url = "https://private-index.example.com"
priority = "supplementary"
```

Credentials would be set with the following environment variables:

- `POETRY_HTTP_BASIC_FOO_BAR_USERNAME`
- `POETRY_HTTP_BASIC_FOO_BAR_PASSWORD`

For uv, this would translate to:

- `UV_INDEX_FOO_BAR_USERNAME`
- `UV_INDEX_FOO_BAR_PASSWORD`

To forward those credentials to `migrate-to-uv`, you can either export them beforehand, or set the environment variables
when invoking the command:

```bash
# Either
export UV_INDEX_FOO_BAR_USERNAME=<username>
export UV_INDEX_FOO_BAR_PASSWORD=<password>
migrate-to-uv

# Or
UV_INDEX_FOO_BAR_USERNAME=<username> \
  UV_INDEX_FOO_BAR_PASSWORD=<password> \
  migrate-to-uv
```
