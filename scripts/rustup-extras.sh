#!/usr/bin/env bash
set -euo pipefail
if [[ -n ${CARGO_BUILD_TARGET:-} ]]
then
    rustup target add $CARGO_BUILD_TARGET --toolchain $TRAVIS_RUST_VERSION
fi
