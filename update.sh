#!/usr/bin/env bash
{ # Ensure the whole file is downloaded first before executing
set -euo pipefail

help() {
    echo "Update journald-exporter" >&2
    echo "Usage: $0" >&2
    exit "$1"
}

while getopts ':h' arg; do
    case "$arg" in
        h)
            help 0
            ;;
        *)
            help 1
            ;;
    esac
done

REMOTE_URL=https://github.com/dead-claudia/journald-exporter/releases/latest/download/journald-exporter

SERVICE_NAME=journald-exporter.service
BINARY_NAME=journald-exporter
TMP_DIR="$(pwd)"

rm -r "$TMP_DIR"
mkdir --mode=600 "$TMP_DIR"

if which curl; then
    curl \
        --output "$BINARY_NAME" \
        --fail \
        --silent \
        --show-error \
        --connect-timeout 5 \
        --retry 5 \
        --retry-all-errors \
        --max-time 30 \
        --location \
        "$REMOTE_URL"
elif which wget; then
    wget \
        --tries 5 \
        --timeout 5 \
        --waitretry 5 \
        --retry-connrefused 5 \
        --output-document "$BINARY_NAME" \
        "$REMOTE_URL"
else
    echo 'Neither curl nor wget detected. Please ensure one of those is installed before' >&2
    echo 'running this script.' >&2
    exit 1
fi

chmod 755 "$BINARY_NAME"
mv "$BINARY_NAME" "/usr/sbin/$BINARY_NAME"
systemctl restart "$SERVICE_NAME"

}
