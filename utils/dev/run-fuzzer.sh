#!/bin/sh

set -ea

command -V cargo

projRoot=$(git rev-parse --show-toplevel)

cd "$projRoot"/libs/core
. ../../containers/local.env

cargo test --test exhaustive_sync_check --profile=release-lto --features 'no-network' -- --nocapture --ignored
