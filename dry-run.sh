#!/bin/bash

set -euo pipefail

cargo run --release -- \
  --keystore-url unix:///Users/connor/code/sprillow/holochain-runner/test/socket?k=RaoxVOqYYYRmILJrG5c9fAdFOWcyQDW83pLEEL0pJRI \
  test/test.happ test/data/databases
#cargo run --release -- --keystore-path test/data/keystore test/hrea_suite.happ test/data/databases
