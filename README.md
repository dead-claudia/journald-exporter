# journald-exporter

[![CI](https://github.com/dead-claudia/journald-exporter/actions/workflows/ci.yml/badge.svg)](https://github.com/dead-claudia/journald-exporter/actions/workflows/ci.yml)

This is all written in Rust for simplicity and performance.

- [Installation and updating](#installation-and-updating)
- [Contributing](#contributing)
- [Metrics emitted](#metrics-emitted)
- [License](#license)

## Installation and updating

See [the installation guide](installation.md). Be sure to also save a link to this README in your runbook, in the rare event things go wrong. While this is extensively tested and tries its best to handle all reasonable error modes, no software is perfect, and this is no exception.

## Contributing

See [the contributing guide](CONTRIBUTING.md). It also documents the internals at a high level. Consider checking it out if you want to contribute!

## Metrics emitted

- Counter `journald_entries_ingested`: The total number of entries ingested.
- Counter `journald_fields_ingested`: The total number of data fields read. Normally it's 5 fields (`_SYSTEMD_UNIT`, `PRIORITY`, `_UID`, `_GID`, and `MESSAGE`) for message entries, but may be fewer in the case of incomplete fields and such.
- Counter `journald_data_ingested_bytes`: The total number of data field bytes ingested across all fields, including both keys and their values.
- Counter `journald_faults`: The total number of faults encountered while reading the journal.
- Counter `journald_cursor_double_retries`: Total number of faults encountered while recovering after a previous fault. Also increments if it fails on first read. Note: too many of these in a short period of time will cause entire program to crash.
- Counter `journald_unreadable_fields`: The total number of fields unreadable for reasons other than being corrupted (usually, too large to be read).
- Counter `journald_corrupted_fields`: The total number of corrupted entries detected while reading the journal that could still be read.
- Counter `journald_metrics_requests`: The total number of requests received, including requests to paths other than the standard `GET /metrics` route.
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
