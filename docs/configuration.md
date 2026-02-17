---
icon: lucide/wrench
---
# Configuration

## Project path

By default, `migrate-to-uv` uses the current directory to search for the project to migrate. If the project is in a
different path, you can set the path to a directory as a positional argument.

**Example**:

```bash
# Relative path
migrate-to-uv subdirectory

# Absolute path
migrate-to-uv /home/foo/project
```

## Arguments

`migrate-to-uv` provides a few arguments to let you customize how the migration is performed.

### `--dry-run`

Run the migration without modifying the files, printing the changes that would have been made in the terminal instead.

**Example**:

```bash
migrate-to-uv --dry-run
```

### `--skip-lock`

By default, `migrate-to-uv` locks dependencies with `uv lock` at the end of the migration. This flag disables this
behavior.

**Example**:

```bash
migrate-to-uv --skip-lock
```

### `--skip-uv-checks`

By default, `migrate-to-uv` will exit early if a project already uses uv. This flag disables this behavior, allowing
`migrate-to-uv` to run on a `pyproject.toml` that already has uv configured.

Note that the project must also have a valid non-uv package manager configured, otherwise it will fail to generate the
uv configuration.

**Example**:

```bash
migrate-to-uv --skip-uv-checks
```

### `--ignore-locked-versions`

By default, when locking dependencies with `uv lock`, `migrate-to-uv` keeps dependencies to the versions they were
locked to with the previous package manager, if it supports lock files, and if a lock file is found. This behavior can
be disabled, in which case dependencies will be locked to the highest possible versions allowed by the dependencies
constraints.

**Example**:

```bash
migrate-to-uv --ignore-locked-versions
```

### `--replace-project-section`

By default, existing data in `[project]` section of `pyproject.toml` is preserved when migrating. This flag allows
completely replacing existing content.

**Example**:

```bash
migrate-to-uv --replace-project-section
```

### `--package-manager`

By default, `migrate-to-uv` tries to auto-detect the package manager based on the files (and their content) used by the
package managers it supports. If auto-detection does not work in some cases, or if you prefer to explicitly specify the
package manager, you can explicitly set it.

**Available options**:

- `pip`
- `pip-tools`
- `pipenv`
- `poetry`

**Example**:

```bash
migrate-to-uv --package-manager poetry
```

### `--build-backend`

The build backend to choose when performing the migration. If the option is not provided, the build backend will be
automatically chosen based on the package distribution complexity,
using [uv](https://docs.astral.sh/uv/concepts/build-backend/) if it is simple enough, or
using [Hatch](https://hatch.pypa.io/latest/config/build/) otherwise.

!!!note

    If you explicitly choose `uv` and the migration cannot be performed because the project uses package distribution
    metadata that cannot be expressed with uv build backend, the migration will fail, suggesting to use hatch with
    `--build-backend hatch`.

**Available options**:

- `hatch`
- `uv`

**Example**:

```bash
migrate-to-uv --build-backend uv
```

### `--dependency-groups-strategy`

Most package managers that support dependency groups install dependencies from all groups when performing installation.
By default, uv will [only install `dev` one](https://docs.astral.sh/uv/concepts/projects/dependencies/#default-groups).

In order to match the current package manager as closely as possible, `migrate-to-uv` defaults to setting
`default-groups = "all"` under `[tool.uv]` section, unless a dependency group is set as optional (like
[Poetry allows to do](https://python-poetry.org/docs/managing-dependencies#optional-groups)), in which case it defaults to explicitly setting non-optional groups to `default-groups` (e.g., `default-groups = ["dev", "typing"]`).

If the default behavior is not suitable, it is possible to change it.

**Available options**:

- `set-default-groups-all`: Move each dependency group to its corresponding uv dependency group, and set
  `default-groups = "all"` under `[tool.uv]` section to automatically select all dependency groups by default
- `set-default-groups`: Move each dependency group to its corresponding uv dependency group, and add all
  non-optional dependency groups in `default-groups` under `[tool.uv]` section (unless the only dependency group is
  `dev` one, as this is already uv's default)
- `include-in-dev`:  Move each dependency group to its corresponding uv dependency group, and reference all non-optional
  dependency groups (others than `dev` one) in `dev` dependency group by using `{ include-group = "<group>" }`
- `keep-existing`: Move each dependency group to its corresponding uv dependency group, without any further action
- `merge-into-dev`: Merge dependencies from all non-optional dependency groups into `dev` dependency group (optional
  dependency groups are moved to their corresponding uv dependency groups)

**Example**:

```bash
migrate-to-uv --dependency-groups-strategy include-in-dev
```

### `--requirements-file`

Names of the production requirements files to look for, for projects using `pip` or `pip-tools`. The argument can be set
multiple times, if there are multiple files.

**Example**:

```bash
migrate-to-uv --requirements-file requirements.txt --requirements-file more-requirements.txt
```

### `--dev-requirements-file`

Names of the development requirements files to look for, for projects using `pip` or `pip-tools`. The argument can be
set multiple times, if there are multiple files.

**Example**:

```bash
migrate-to-uv --dev-requirements-file requirements-dev.txt --dev-requirements-file requirements-docs.txt
```

### `--keep-current-build-backend`

Keep the current build backend during the migration. This can be useful if the build backend cannot be expressed with
any of the supported build backends (Hatch and uv), if you prefer to migrate it yourself, or if you want to stay on the
current build backend.

When using Poetry (which is the only package manager where this setting applies), this will:

- leave `[build-system]` section untouched
- keep [`packages`](https://python-poetry.org/docs/pyproject#packages) and [`include`/`exclude`](https://python-poetry.org/docs/pyproject#exclude-and-include) keys from `[tool.poetry]` section

**Example**:

```bash
migrate-to-uv --keep-current-build-backend
```

### `--keep-current-data`

Keep the current package manager data (lock file, sections in `pyproject.toml`, ...) after the migration, if you want to
handle the cleaning yourself, or want to compare the differences first.

**Example**:

```bash
migrate-to-uv --keep-current-data
```

### `--ignore-errors`

Perform the migration even if errors occur during the process, which would likely lead to a partial migration. Errors
will still be displayed in the terminal. This flag could be useful if you prefer to manually fix the migration
afterward based on the errors that could occur during the migration.

**Example**:

```bash
migrate-to-uv --ignore-errors
```
