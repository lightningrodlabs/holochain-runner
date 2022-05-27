#!/bin/bash

set -euo pipefail

cargo run --release -- --keystore-path test/data/keystore test/elemental-chat.happ test/data/databases
#cargo run --release -- --keystore-path test/data/keystore test/hrea_suite.happ test/data/databases
