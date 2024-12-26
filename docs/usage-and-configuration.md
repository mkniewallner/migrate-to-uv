# Usage and configuration

## Basic usage

```bash
# With uv
uvx migrate-to-uv

# With pipx
pipx run migrate-to-uv
```

## Configuration

### Project path

By default, `migrate-to-uv` uses the current directory to search for the project to migrate. If the project is in a
different path, you can set the path to a directory as a positional argument, like so:

```bash
# Relative path
migrate-to-uv subdirectory

# Absolute path
migrate-to-uv /home/foo/project
```

### Arguments

While `migrate-to-uv` tries, as much as possible, to match what the original package manager defines for a project
when migrating the metadata to uv, there are features that could be present in a package manager that does not exist in
uv, or behave differently. Mainly for those reasons, `migrate-to-uv` offers a few options.

#### `--dry-run`

This runs the migration, but without modifying the files. Instead, it prints the changes that would have been made in
the terminal.

**Example**:

```bash
migrate-to-uv --dry-run
```

#### `--package-manager`

By default, `migrate-to-uv` tries to auto-detect the package manager based on the files (and their content) used by the
package managers it supports. If auto-detection does not work in some cases, or if you prefer to explicitly specify the
package manager, this option could be used.

**Example**:

```bash
migrate-to-uv --package-manager poetry
```

#### `--dependency-groups-strategy`

Most package managers that support dependency groups install dependencies from all groups when performing installation.
By default, uv will [only install `dev` one](https://docs.astral.sh/uv/concepts/projects/dependencies/#default-groups).

In order to match the workflow in the current package manager as closely as possible, by default, `migrate-to-uv` will
move each dependency group to its corresponding one in uv, and set all dependency groups in `default-groups` under
`[tool.uv]` section (unless the only dependency group is `dev` one, as this is already uv's default).

If this is not desirable, it is possible to change the strategy by using `--dependency-groups-strategy <VALUE>`, where
`<VALUE>` can be one of the following:

- `set-default-groups` (default): Move each dependency group to its corresponding uv dependency group, and add all
  dependency groups in `default-groups` under `[tool.uv]` section (unless the only dependency group is `dev` one, as
  this is already uv's default)
- `include-in-dev`:  Move each dependency group to its corresponding uv dependency group, and reference all dependency
  groups (others than `dev` one) in `dev` dependency group by using `{ include = "<group>" }`
- `keep-existing`: Move each dependency group to its corresponding uv dependency group, without any further action
- `merge-into-dev`: Merge dependencies from all dependency groups into `dev` dependency group

**Example**:

```bash
migrate-to-uv --dependency-groups-strategy include-in-dev
```

#### `--keep-current-data`

Keep the current package manager data (lock file, sections in `pyproject.toml`, ...) after the migration, if you want to
handle the cleaning yourself, or want to compare the differences first.
