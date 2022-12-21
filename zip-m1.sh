#!/bin/bash

if [[ $(uname -m) == 'arm64' ]]; then
  cd target/release && tar -cvzf holochain-runner-arm64-apple-darwin.tar.gz holochain-runner
fi

