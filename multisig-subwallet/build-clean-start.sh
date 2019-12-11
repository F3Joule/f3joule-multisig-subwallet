#!/usr/bin/env bash

./scripts/build-runtime.sh
cargo build
cargo run -- purge-chain --dev
cargo run -- --dev
