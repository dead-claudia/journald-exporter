#!/usr/bin/env bash
set -euo pipefail

# Make sure everything parses fully before running. Easier to debug that way
{

# Limit the test duration to 1 minute by default, but leave it configurable for local testing.
test_duration=60
port=8080
binary="$(pwd)/target/release/journald-exporter"
type="http"

bail() {
    echo "$@" >&2
    exit 1
}

while getopts ':p:d:b:t:' opt; do
    case "$opt" in
        p)
            [[ "$OPTARG" =~ ^[[:digit:]]+$ ]] || bail 'Port must be an integer if provided.'
            (( port = OPTARG ))
            (( port >= 1 && port <= 65535 )) || bail 'Port must be within 1 and 65535 inclusive.'
            ;;
        d)
            [[ "$OPTARG" =~ ^[[:digit:]]+$ ]] || bail 'Test duration must be an integer if provided.'
            (( test_duration = OPTARG ))
            (( test_duration >= 1 && test_duration % 1 == 0 )) || bail 'Test duration must be a positive number of seconds.'
            ;;
        b)
            [[ -f "$OPTARG" ]] || bail 'Release binary must be a file.'
            binary="$(readlink -f "$OPTARG")"
            ;;
        t)
            case "$OPTARG" in
                http) type=http ;;
                https) type=https ;;
                *) bail 'Type arg must be either "http" or "https".'
            esac
            ;;
        *)
            bail "Unknown option '$OPTARG'."
            ;;
    esac
done

(( $(id -u) == 0 )) || bail 'This script must run as root.'

TEST_INTERVAL=5
REQUEST_TIMEOUT=5

API_KEY_DIR=/tmp/integ-test.keys
TLS_PUBLIC_CERT=/tmp/integ-test-cert.pem
TLS_PRIVATE_KEY=/tmp/integ-test-key.pem
CURL_CONFIG=/tmp/integ-test-curl.conf

[[ -d "$API_KEY_DIR" ]] || bail "API key directory missing. Did you forget to run 'scripts/e2e-setup.sh' first?"
[[ -f "$TLS_PUBLIC_CERT" ]] || bail "TLS public certificate missing. Did you forget to run 'scripts/e2e-setup.sh' first?"
[[ -f "$TLS_PRIVATE_KEY" ]] || bail "TLS private key missing. Did you forget to run 'scripts/e2e-setup.sh' first?"

action_type=
journalctl_job=
unit=

stop() {
    [[ -n "$unit" ]] && {
        systemctl stop "$unit"
        echo '[INTEG] Child stopped' >&2
        unit=
    }

    [[ -n "$journalctl_job" ]] && {
        kill "$journalctl_job"
        journalctl_job=
    }
}

trap stop EXIT ERR TERM INT

tty="$(tty)"
start_lines=()

echo "user = metrics:$(cat "$API_KEY_DIR/test.key")" > $CURL_CONFIG

extra_args=()
[[ $type == https ]] && extra_args=(--certificate "$TLS_PUBLIC_CERT" --private-key "$TLS_PRIVATE_KEY")

mapfile -t start_lines < <(
    systemd-run \
    --collect \
    --property='Type=notify' \
    --property='WatchdogSec=10s' \
    --property='TimeoutStartSec=10s' \
    "$binary" \
    --port $port \
    --key-dir $API_KEY_DIR \
    "${extra_args[@]}" 2>&1 |
    tee "$tty"
)

for line in "${start_lines[@]}"; do
    if [[ "$line" =~ 'Running as unit: '([A-Za-z0-9@_-]+'.service') ]]; then
        unit="${BASH_REMATCH[1]}"
        action_type=fetch
    elif [[ "$line" =~ 'Job for '([A-Za-z0-9@_-]+'.service')' failed' ]]; then
        unit="${BASH_REMATCH[1]}"
        action_type=fail
    fi
done

if [[ "$action_type" == fetch ]]; then
    echo "[INTEG] Detected transient unit name: $unit" >&2
    journalctl --unit="$unit" --follow --since=-1d --output=cat &
    journalctl_job=$!

    # Give the server time to boot and initial journal entries time to display.
    sleep 5

    echo "[INTEG] Starting fetch loop" >&2

    current_time=$(date +%s)
    (( fetch_stop_time = current_time + test_duration ))

    while (( current_time < fetch_stop_time )); do
        start_time=$(date +%s)

        read -r output < <(
            if [[ $type == http ]]; then
                curl \
                    --config $CURL_CONFIG \
                    --max-time $REQUEST_TIMEOUT \
                    --write-out '[INTEG] Response: %{response_code} %{content_type} %{size_download}B\n' \
                    --output /dev/null \
                    --fail \
                    --silent \
                    --show-error \
                    http://localhost:$port/metrics
            else
                curl \
                    --config $CURL_CONFIG \
                    --max-time $REQUEST_TIMEOUT \
                    --write-out '[INTEG] Response: %{response_code} %{content_type} %{size_download}B\n' \
                    --output /dev/null \
                    --fail \
                    --silent \
                    --show-error \
                    --cacert $TLS_PUBLIC_CERT \
                    --insecure \
                    https://localhost:$port/metrics
            fi | tee "$tty"
        )

        [[ "$output" =~ 'Response: 0' ]] && bail '[INTEG] Request failed.'
        [[ "$output" =~ 'application/openmetrics-text '[0-9]+'B' ]] || bail '[INTEG] Received wrong content type.'
        [[ "$output" =~ 'application/openmetrics-text 0B' ]] && bail '[INTEG] Received empty response.'

        # Now, sleep to the next request time.
        end_time=$(date +%s)
        (( sleep_interval = TEST_INTERVAL - ( end_time - start_time ) ))
        sleep $(( sleep_interval < 0 ? 0 : sleep_interval ))
        current_time=$(date +%s)
    done
elif [[ "$action_type" == fail ]]; then
    echo "[INTEG] Unit failed to initialize: $unit" >&2
    journalctl --unit="$unit" --catalog --output=cat
    systemctl status "$unit"
else
    bail "[INTEG] Unknown action type: '$action_type'"
fi

}
