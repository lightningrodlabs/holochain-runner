#!/bin/bash

set -euo pipefail

# when running this command, pass the passphrase like this:
# echo mypassword | ./dry-run.sh

RUST_LOG=debug cargo run --release -- \
  --keystore-path ~/code/holochain-runner/test/data/keystore \
  test/test.happ test/data/databases
#cargo run --release -- --keystore-path test/data/keystore test/hrea_suite.happ test/data/databases
