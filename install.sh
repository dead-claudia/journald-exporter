#!/usr/bin/env bash
{ # Ensure the whole file is downloaded first before executing
set -euo pipefail

# Placed first to also help script readers a bit so they don't have to look so hard.
help() {
    cat >&2 <<EOF
Usage: $0 [ -p <port> ] [ -k <key-file> ]

Arguments:

-p PORT
    The port to expose the to-be-installed metrics server from.

-k KEY_FILE
    A pre-made key file to pre-install when setting up the server. This can be
    specified multiple times.

Copyright 2023 Claudia Meadows

Licensed under the Apache License, Version 2.0 (the "License"); you may not
use this file except in compliance with the License. You may obtain a copy
of the License at <http://www.apache.org/licenses/LICENSE-2.0>.

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
License for the specific language governing permissions and limitations
under the License.

Source code for the latest version at the time of writing can be found at
<https://github.com/dead-claudia/journald-exporter>.
EOF
    exit "$1"
}

main() {
    local -i port=12345
    local -a key_files=()

    while getopts ':p:k:h' arg; do
        case "$arg" in
            p)
                [[ "$OPTARG" =~ ^[[:digit:]]*$ ]] || fail 'Port must be an integer'
                port=$OPTARG
                [[ $port -gt 0 && $port -le 65535 ]] || fail "Port $port is out of range"
                ;;
            k)
                [[ -f "$OPTARG" ]] || fail 'Key file must exist'
                key_files+=("$OPTARG")
                ;;
            h)
                help 0
                ;;
            *)
                help 1
                ;;
        esac
    done

    check-systemd-journald-running

    local tmp_dir key_name client_type
    tmp_dir="$(cd ./journald-exporter-install; pwd)"
    key_name="$(date -u +'%Y-%m-%dT%H:%M:%SZ.key')"
    client_type="$(get-fetch-command)"

    rm -r "$tmp_dir"
    mkdir --mode=700 "$tmp_dir"

    fetch-binary "$client_type" "$tmp_dir/journald-exporter"
    chmod 755 "$tmp_dir/journald-exporter"
    prepare-service-file "$tmp_dir/journald-exporter.service" "$port"
    prepare-keys-dir "$tmp_dir/etc-journald-exporter" "$key_name" "${key_files[@]}"

    useradd --system --user-group journald-exporter
    mv "$tmp_dir/journald-exporter" /usr/sbin/journald-exporter
    mv "$tmp_dir/etc-journald-exporter" /etc/journald-exporter
    mv "$tmp_dir/journald-exporter.service" /etc/systemd/system/journald-exporter.service

    systemd-analyze verify "$tmp_dir/journald-exporter.service" || \
        bug 'Generated service file for journald-exporter.service is invalid.'

    systemctl daemon-reload
    systemctl start journald-exporter.service

    sleep 1
    check-available "$client_type"

    echo "Generated API key located at '/etc/journald-exporter/keys/$key_name'." >&2
}

fail() {
    echo "$1" >&2
    exit 1
}

bug() {
    echo "BUG: $1" >&2
    exit 2
}

get-fetch-command() {
    if which curl >/dev/null 2>&1; then
        echo 'curl'
    elif which wget >/dev/null 2>&1; then
        echo 'wget'
    else
        fail 'Neither curl nor wget detected. Please ensure one of those is installed before
running this script.'
    fi
}

check-systemd-journald-running() {
    [[ -e /run/systemd/journal/socket ]] || \
        fail "This system is not running systemd-journald. Either ensure both systemd and
systemd-journald is running, or consider seeking an alternative solution as
this tool is probably not the droid you're looking for. :-)"
}

fetch-binary() {
    local client_type="$1"
    local output_file="$2"
    local remote_url=https://github.com/dead-claudia/journald-exporter/releases/latest/download/journald-exporter

    case $client_type in
        curl)
            curl \
                --output "$output_file" \
                --fail \
                --silent \
                --show-error \
                --connect-timeout 5 \
                --retry 5 \
                --retry-all-errors \
                --retry-max-time 15 \
                --max-time 60 \
                --location \
                $remote_url
            ;;

        wget)
            wget \
                --tries=5 \
                --timeout=5 \
                --waitretry=5 \
                --retry-connrefused \
                --output-document="$output_file" \
                $remote_url
            ;;

        *) bug "Invalid client type: $client_type" ;;
    esac
}

get-systemd-version() {
    systemctl --version | sed -n 's/^systemd \([0-9]\+\).*/\1/p'
}

prepare-service-file() {
    local output_file="$1"
    local port="$2"

    cat >"$output_file" <<EOF
[Unit]
Description=journald-exporter
Documentation=https://github.com/dead-claudia/journald-exporter
# Couple conditions so it doesn't immediately bork on startup. The program also
# checks for the directory, but this avoids having to reset the failure counter
# in case it fails for whatever reason.
After=network.target
# Asserting here as it's pretty important to make sure metrics are flowing.
AssertPathIsDirectory=/etc/journald-exporter/keys

# So it'll run on startup.
[Install]
WantedBy=default.target

[Service]
Type=notify
ExecStart=/usr/sbin/journald-exporter --key-dir /etc/journald-exporter/keys --port $port
WatchdogSec=5m
Restart=always
# And a number of security settings to lock down the program somewhat.
NoNewPrivileges=true
ProtectSystem=strict
ProtectClock=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectKernelLogs=true
ProtectControlGroups=true
MemoryDenyWriteExecute=true
SyslogLevel=warning
SyslogLevelPrefix=false
EOF
}

prepare-keys-dir() {
    local target="$1"
    local key_name="$2"
    shift 2
    mkdir --mode=755 "$target"
    mkdir --mode=755 "$target/keys"

    if [[ "${#@}" -eq 0 ]]; then
        cp -t "$target/keys" "${@}"
    else
        openssl rand -hex -out "$target/keys/$key_name"
    fi
    chmod 600 "$target/keys/$key_name"
}

check-available() {
    local client_type="$1"
    local tmp_dir="$2"
    local key_name="$3"
    local -i port="$4"
    local test_config="$5"

    local test_endpoint=http://localhost:$port/metrics

    case $client_type in
        curl)
            (echo -n '--user=metrics:'; cat "$key_name") > "$test_config"
            curl \
                --config "$test_config" \
                --output /dev/null \
                --fail \
                --silent \
                --connect-timeout 5 \
                --retry 10 \
                --retry-max-time 15 \
                --retry-all-errors \
                --max-time 120 \
                $test_endpoint
            ;;

        wget)
            (echo 'user=metrics'; echo -n 'password='; cat "$key_name") > "$test_config"
            wget \
                --config "$test_config" \
                --quiet \
                --tries=12 \
                --timeout=5 \
                --waitretry=10 \
                --retry-connrefused \
                --output-document="$tmp_dir/journald-exporter" \
                $test_endpoint
            ;;

        *) bug "Invalid client type: $client_type" ;;
    esac || fail 'Service not successfully initialized.'
}

main

}
