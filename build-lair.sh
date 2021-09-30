#!/bin/bash

set -euo pipefail

tag="${GITHUB_REF#refs/tags/}"

cargo install lair_keystore --root . --git https://github.com/guillemcordoba/lair --rev 53fea43d53352609978d550f72f987f0ce458896

gh release upload "${tag}" "lair_keystore" --clobber
