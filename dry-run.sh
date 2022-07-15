#!/bin/bash

set -euo pipefail

cargo run --release -- --keystore-path test/data/keystore test/test.happ test/data/databases
#cargo run --release -- --keystore-path test/data/keystore test/hrea_suite.happ test/data/databases
