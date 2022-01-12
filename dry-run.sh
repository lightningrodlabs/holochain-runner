#!/bin/bash

set -euo pipefail

# cargo run --manifest-path crates/holochain-runner/Cargo.toml --release -- --keystore-path test/data/keystore test/profiles.dna test/data/databases
cargo run --manifest-path crates/holochain-runner/Cargo.toml -- --keystore-path test/data/keystore test/profiles.dna test/data/databases
