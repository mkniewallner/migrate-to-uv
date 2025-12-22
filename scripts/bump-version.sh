#!/usr/bin/env bash

set -euo pipefail

version=$1

sed -i "s/^version = \".*\"/version = \"${version}\"/" Cargo.toml pyproject.toml
sed -i "s/^## Unreleased/## ${version} - $(date +%F)/" CHANGELOG.md
./scripts/copy-changelog.sh
cargo update migrate-to-uv
uv lock --upgrade-package migrate-to-uv
