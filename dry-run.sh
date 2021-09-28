#!/bin/bash

set -euo pipefail

cargo run -- --keystore-path test/data/keystore test/profiles.dna test/data/databases
