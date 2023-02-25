#!/usr/bin/env bash
set -euo pipefail

DISABLE_ISOLATION=

while getopts ':b' opt; do
    case "$opt" in
        b)
            DISABLE_ISOLATION=1
            ;;
        *)
            echo "Unknown option '$OPTARG'." >&2
            exit 1
            ;;
    esac
done

cargo clean

if [[ -n "$DISABLE_ISOLATION" ]]; then
    export RUST_BACKTRACE=1
    export MIRIFLAGS="${MIRIFLAGS:-} -Zmiri-disable-isolation"
else
    echo "Note: run './scripts/miri-test.sh -b' to get backtraces."
fi

exec cargo +nightly miri test "$@"
