#!/bin/bash

if [[ $(uname -m) == 'arm64' ]]; then
  cargo build --release --features sqlite-encrypted
  cd target/release && tar -cvzf holochain-runner-arm64-apple-darwin.tar.gz holochain-runner
  echo "output: target/release/holochain-runner-arm64-apple-darwin.tar.gz"
else
  echo "This script is only for Apple Silicon"
fi

