#!/bin/bash

set -euo pipefail

tag="${GITHUB_REF#refs/tags/}"

cargo install lair_keystore --root . --version 0.0.3

gh release upload "${tag}" "lair_keystore" --clobber
