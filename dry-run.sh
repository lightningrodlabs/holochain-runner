#!/bin/bash

set -euo pipefail

# first, separately, run
# `lair-server init`
# set a passphrase
# now, run:
# `lair-keystore server`
# input the passphrase
# take the connection url from there,
# and put it below instead of `add-connection-url-here`

# when running this command, pass the passphrase like this:
# echo mypassword | ./dry-run.sh

cargo run --release -- \
  --keystore-url add-connection-url-here \
  test/test.happ test/data/databases
#cargo run --release -- --keystore-path test/data/keystore test/hrea_suite.happ test/data/databases
