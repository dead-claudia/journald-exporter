# journald-exporter

[![CI](https://github.com/dead-claudia/journald-exporter/actions/workflows/ci.yml/badge.svg)](https://github.com/dead-claudia/journald-exporter/actions/workflows/ci.yml)

This is all written in Rust for simplicity and performance.

- [Installation](#installation)
- [Metrics emitted](#metrics-emitted)
- [License](#license)

## Installation

1. [Download the latest binary](https://github.com/dead-claudia/journald-exporter/releases) and install it somewhere in your `$PATH`. `/usr/sbin/journald-exporter` is recommended.

2. Set up a `journald-exporter` system user with same-named group. On Debian, `sudo adduser --system --group journald-exporter` will work. (No home directory is needed.)

3. Place a systemd service unit file in `/usr/local/lib/systemd/system/journald-exporter.service` with the following contents:

    ```ini
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
    ExecStart=/usr/sbin/journald-exporter --path /etc/journald-exporter/keys --port 12345
    WatchdogSec=5m
    Restart=always
    # And a number of security settings to lock down the program as best as
    # reasonably possible.
    NoNewPrivileges=true
    ProtectSystem=strict
    NoExecPaths=/
    ExecPaths=/usr/sbin/journald-exporter
    ProtectClock=true
    ProtectKernelTunables=true
    ProtectKernelModules=true
    ProtectKernelLogs=true
    ProtectControlGroups=true
    MemoryDenyWriteExecute=true
    SyslogLevel=warning
    SyslogLevelPrefix=false
    ```

    If you want, or need, to change the key directory or port number, this is where you'll need to configure it.

4. Make the `/etc/journald-exporter/keys` directory mentioned above. This is where you'll put your API keys.

5. Create an API key via `openssl rand -hex -out "$(date -uIseconds).key" 32` and copy it to `/etc/journald-exporter/keys`.

6. Start the service (as in, `sudo systemctl start journald-exporter.service`).

7. If you plan to listen over the public Internet, set up a local TLS proxy that accepts encrypted connections and forwards them decrypted to the server to avoid leaking the API key. Then, you can poke holes as needed in your system's firewall to allow inbound connections to the above port. If you plan to only invoke this locally (say, you're using [Grafana Agent](https://grafana.com/docs/agent/latest/)), you can skip this step (but do *not* open that port in the system firewall).

8. [Configure your Prometheus instance to scrape the service](https://prometheus.io/docs/prometheus/latest/configuration/configuration/#scrape_config) using the API key file generated from earlier as the password file. If you're using [Grafana Agent](https://grafana.com/docs/agent/latest/) or similar, you'll want to add the scrape config rule there instead. If you set up a TLS proxy to run it over the public Internet, use the port you set that up with, not the one you configured this exporter to use.

9. If you've done everything above, metrics should start flowing.

To update, just replace the binary (you'll want to use the `mv journald-exporter /usr/sbin/` file replacement trick to avoid "text file busy" file access errors) and restart the service via `systemctl restart journald-exporter.service`.

## Metrics emitted

- Counter `journald_entries_ingested`: The total number of entries ingested.
- Counter `journald_fields_ingested`: The total number of data fields read. Normally it's 5 fields (`_SYSTEMD_UNIT`, `PRIORITY`, `_UID`, `_GID`, and `MESSAGE`) for message entries, but may be fewer in the case of incomplete fields and such.
- Counter `journald_data_bytes_ingested`: The total number of data field bytes ingested across all fields, including both keys and their values.
- Counter `journald_faults`: The total number of faults encountered while reading the journal.
- Counter `journald_cursor_double_retries`: Total number of faults encountered while recovering after a previous fault. Also increments if it fails on first read. Note: too many of these in a short period of time will cause entire program to crash.
- Counter `journald_unreadable_fields`: The total number of fields unreadable for reasons other than being corrupted (usually, too large to be read).
- Counter `journald_corrupted_fields`: The total number of corrupted entries detected while reading the journal that could still be read.
- Counter `journald_metrics_requests`: The total number of metrics requests received.
  - This can also be used to ensure the server's live and receiving requests.
  - This can also be used to ensure that anything like [Grafana Agent](https://grafana.com/docs/agent/latest/) is in fact scraping metrics at the desired frequency, and if done locally, it can isolate that very easily from network malfunctions.
- Counter `journald_messages_ingested`: Number of message entries successfully processed.
- Counter `journald_messages_ingested_bytes`: Total number of `MESSAGE` field bytes ingested.

The `journald_messages_ingested` and `journald_messages_ingested_bytes` metrics include a few extra dimensions to allow more in-depth inspection:

- Key `service`: A systemd service name, usually ending in `.service`, or `?` if no service is present.
- Key `priority`: A syslog priority keyword in uppercase. If none can be discerned, `ERR` is used.
  - `priority="EMERG"`
  - `priority="ALERT"`
  - `priority="CRIT"`
  - `priority="ERR"` (also used in case of missing priority)
  - `priority="WARNING"`
  - `priority="NOTICE"`
  - `priority="INFO"`
  - `priority="DEBUG"`
- Key `severity`: A syslog severity number, an integer from 0 to 7 inclusive. Corresponds with `priority`.
- Key `user`: The name of the active user as per the data point given, the UID if it could not be discerned, or `?` if no UID is present.
- Key `group`: The name of the active group as per the data point given, the GID if it could not be discerned, or `?` if no GID is present.

To ensure global `sum` works, the above two metrics return a simple unlabeled 0 if no entries have been added yet.

## License

Copyright 2023 Claudia Meadows

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at <http://www.apache.org/licenses/LICENSE-2.0> or in the LICENSE.txt file of this directory.

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
