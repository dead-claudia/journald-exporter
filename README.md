# journald-exporter

[![CI](https://github.com/dead-claudia/journald-exporter/actions/workflows/ci.yml/badge.svg)](https://github.com/dead-claudia/journald-exporter/actions/workflows/ci.yml)

This is all written in Rust for simplicity and performance.

- [Installation](#installation)
- [Metrics emitted](#metrics-emitted)
- [License](#license)

## Installation

> This assumes a basic understanding of shell commands and shell variables.

1. [Download the latest binary](https://github.com/dead-claudia/journald-exporter/releases) and install it somewhere in your `$PATH`. `/usr/sbin/journald-exporter` is recommended.

    ```sh
    # Assumes `journald-exporter` is in the current directory and `/usr/sbin`
    # is in your default `$PATH`

    # Note: you have to use `mv` to replace the file to avoid "text file busy"
    # file access errors on update
    chmod +x journald-exporter
    sudo cp journald-exporter journald-exporter.tmp
    sudo mv journald-exporter.tmp /usr/sbin/journald-exporter
    ```

2. Set up a `journald-exporter` system user with same-named group. No home directory is needed.

    ```sh
    sudo useradd --system --user-group journald-exporter
    ```

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

4. Make the `/etc/journald-exporter/keys` directory mentioned above, with an owner of root and permissions of 755 (user has all perms, everyone else has only read/execute). This is where you'll put your API keys.

    ```sh
    sudo mkdir --mode=755 /etc/journald-exporter/keys
    ```

5. Create an API key via `openssl rand -hex -out "$(date -uIseconds).key" 32` and copy it to `/etc/journald-exporter/keys` with an owner of root and permissions of 600 (root can read and write, nobody else can access).

    ```sh
    name="$(date -uIseconds).key"
    sudo openssl rand -hex -out "$name" 32
    sudo chmod 600 "$name"
    sudo mv "$name" "/etc/journald-exporter/keys/$name"
    ```
    
    You'll also want to copy this key - you'll need it later. You can use the following (using `$name` from above) to set up the key for a local Prometheus instance:

    ```sh
    sudo mkdir /etc/prometheus-keys
    sudo cp "/etc/journald-exporter/keys/$name" "/etc/prometheus-keys/$name"
    sudo chgrp journald-exporter "/etc/prometheus-keys/$name"
    sudo chmod 640 "/etc/prometheus-keys/$name"
    ```

6. Start the service.

    ```sh
    sudo systemctl start journald-exporter.service
    ```

7. **If you're scraping locally, skip this step.** If you plan to listen over the public Internet, you'll need to do a couple things to avoid leaking the API key: Set up a local TLS termination proxy to forward connections decrypted to the exporter, then update your system's firewall to allow inbound connections to the port that TLS proxy listens on.

    If you plan to only scrape locally, or if you're using things like [Grafana Agent](https://grafana.com/docs/agent/latest/), you can skip this step. Be careful to *not* open the port for the exporter in the system firewall in this case.

8. [Configure your Prometheus instance to scrape the service](https://prometheus.io/docs/prometheus/latest/configuration/configuration/#scrape_config) using the API key file generated from earlier as the password file. You'll want these fields set:

    - URL: `http://localhost:12345` if you're querying locally, or `https://<your-host-name>:<tls-proxy-port>` if you're accessing it remotely
        - If you set up a TLS proxy in the previous step, use the port you set that up with, not the one for the exporter.
    - Authorization: Basic auth with user `metrics` and password set to the key generated earlier

    If you're scraping locally from Prometheus or similar (like [Grafana Agent](https://grafana.com/docs/agent/latest/)), your scrape config should look something like this, where `$name` is the value generated from step 5:

    ```yaml
    # Add this to the `scrape_configs:` section of your Prometheus config
    - job_name: journald-exporter
      authorization:
        credentials_file: /etc/prometheus-keys/$name
      static_configs:
      - targets:
        - localhost:12345
    ```
    
    > If you're using a different port, you'll obviously want to change `12345` to that.

Once you've done all of this, metrics should start flowing within a few minutes.

To update, just reinstall the binary per the first installation step and restart the service via `systemctl restart journald-exporter.service`.

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
