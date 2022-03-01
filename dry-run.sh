#!/bin/bash

set -euo pipefail

# cargo run --manifest-path crates/holochain-runner/Cargo.toml --release -- --keystore-path test/data/keystore test/profiles.dna test/data/databases
cargo run --release -- --keystore-path test/data/keystore test/hrea_suite.happ test/data/databases
