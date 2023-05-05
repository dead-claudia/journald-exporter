#!/usr/bin/env bash
set -euo pipefail

binary="${1:-"target/release/journald-exporter"}"

assert-exit-code() {
    local -i expected="$1"
    local flag="$2"
    local file="$3"

    local actual=0
    "$binary" "$flag" "$file" || actual=$?

    if [[ $actual -ne $expected ]]; then
        echo "$binary $flag $file exited with code $actual, expected code $expected" >&2
        exit 1
    fi
}

assert-exit-code 0 -c test-configs/valid-http.toml
assert-exit-code 0 -c test-configs/valid-https.toml
assert-exit-code 1 -c test-configs/invalid.toml
assert-exit-code 0 --check test-configs/valid-http.toml
assert-exit-code 0 --check test-configs/valid-https.toml
assert-exit-code 1 --check test-configs/invalid.toml
