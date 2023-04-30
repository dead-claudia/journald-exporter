#!/usr/bin/env bash
set -euo pipefail

if [[ "$(id -u)" -ne 0 ]]; then
    echo 'This script must be run as root.' >&2
    exit 1
fi

API_KEY_DIR=/tmp/integ-test.keys
TLS_PUBLIC_CERT=/tmp/integ-test-cert.pem
TLS_PRIVATE_KEY=/tmp/integ-test-key.pem

# It's okay if this fails, but only outside of CI.
useradd --system --user-group journald-exporter || {
    useradd_code=$?
    [[ -z "${CI:-}" ]] || exit $useradd_code
    true
}

rm -rf "$API_KEY_DIR" "$TLS_PUBLIC_CERT" "$TLS_PRIVATE_KEY"

mkdir -m 755 "$API_KEY_DIR"

# Test API key, used for both HTTP and HTTPS tests
openssl rand -out "$API_KEY_DIR/test.key" -hex 16
chmod 600 "$API_KEY_DIR/test.key"

# Test certificate pair, used for HTTPS tests
openssl req -x509 \
    -newkey rsa:4096 \
    -sha256 \
    -days 3650 \
    -nodes \
    -subj /CN=localhost \
    -addext subjectAltName=DNS:localhost \
    -keyout "$TLS_PRIVATE_KEY" \
    -out "$TLS_PUBLIC_CERT"
