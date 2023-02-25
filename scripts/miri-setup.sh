#!/usr/bin/env bash
set -euo pipefail

rustup toolchain install nightly --component miri
cargo +nightly miri setup
