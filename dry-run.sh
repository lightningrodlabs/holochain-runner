#!/bin/bash

set -euo pipefail

cargo run --release -- --keystore-path test/data/keystore test/profiles.dna test/data/databases
