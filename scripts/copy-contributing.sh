#!/usr/bin/env bash

set -euo pipefail

printf -- '---\n%s\n---\n%s\n' 'icon: lucide/heart-handshake' "$(cat CONTRIBUTING.md)" > docs/CONTRIBUTING.md
