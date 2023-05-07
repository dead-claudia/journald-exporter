// Extra space between the previous/next prompt and the help text is intentional. I want it to look
// like this so it's a little easier to read:
//
// ```
// $ journald-exporter --help
//
// Usage: journald-exporter PORT KEY_FILE_GROUP:KEY_FILE_NAME
//
// [...]
//
// $ _
// ```
pub static HELP_STRING: &str = concat!(
    "\njournald-exporter v",
    env!("CARGO_PKG_VERSION"),
    r#"

Usage:
    journald-exporter --port PORT --key-dir KEY_DIRECTORY
    journald-exporter --config CONFIG_FILE
    journald-exporter --check CONFIG_FILE

Arguments:

-h, -?, -help, --help
    Print this help info. This is also done when no arguments are given. (It's
    pretty lax since if you're wanting help, you probably don't even know how
    to invoke it.)

-v, -V, -version, --version
    Print this utility's version info. (It's lax so you don't have to think
    too hard about how to get it.)

-p PORT, --port PORT
    The port to expose the metrics server from.

-k KEY_DIRECTORY, --key-dir KEY_DIRECTORY
    The directory with accepted API keys.

-C CERTIFICATE_FILE, --certificate CERTIFICATE_FILE
    The PEM-encoded file with the list of public certificates to use for
    HTTPS. Must be used in conjunction with `-K`/`--private-key`.

-K PRIVATE_KEY_FILE, --private-key PRIVATE_KEY_FILE
    The PEM-encoded file with the private key to use for HTTPS. Must be used
    in conjunction with `-C`/`--certificate`.

--config CONFIG_FILE
    A config file to use. Other operands are ignored in this case.

-c CONFIG_FILE, --check CONFIG_FILE
    A config file to check. Other operands are ignored in this case.

Notes:

  - Except for `--check`, the program must be running as root.

  - A `journald-exporter` user is expected to exist, and the web server is
    opened and run under that user.

  - The server exposes a single `/metrics` endpoint that returns metrics.
    Authorization uses the HTTP basic authorization protocol, with a user of
    `metrics` and a password that's one of the accepted API keys. The endpoint
    is rate-limited to one request per second per source IP, and it does not
    attempt to inspect either of the `Forwarded` or `X-Forwarded-For` headers
    to determine the "true" client IP.

  - The key directory is watched, so new API keys can be added and removed
    without having to restart the server. It can also have multiple key files,
    in which all keys in them are accepted, allowing for zero downtime key
    updates. This is only for the key directory - the HTTPS certificate and
    private key files cannot be updated this way.

  - API keys are specified in hex, both within the key files and as the
    "password" for authorization.

License:

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
"#
);

pub static VERSION_STRING: &str = concat!("journald-exporter version v", env!("CARGO_PKG_VERSION"));
