[*Up*](README.md)

# Installation

I've listed flows for scrapers I'm most familiar with. If you want to list out others, please feel free to file a pull request!

- [Install](#install)
  - [Grafana Agent](#grafana-agent)
  - [Local Prometheus](#local-prometheus)
  - [Remote Prometheus](#remote-prometheus)
  - [Other local](#other-local)
  - [Other remote](#other-remote)
  - [Manual](#manual)
- [Update](#update)
  - [Updating the binary](#updating-the-binary)
  - [Updating certificates](#updating-certificates)

## Install

### Grafana Agent

1. Run the installer.

    ```sh
    curl https://raw.githubusercontent.com/dead-claudia/journald-exporter/main/install.sh | sudo bash -s -g grafana-agent
    ```

2. [Configure your Grafana Agent to scrape the service](https://prometheus.io/docs/prometheus/latest/configuration/configuration/#scrape_config) using the API key file generated from earlier as the password file.

    ```yaml
    # Add this to the `scrape_configs:` section of your Prometheus config
    - job_name: journald-exporter
      basic_auth:
        username: metrics
        password_file: /etc/prometheus-keys/journald-exporter.key
      static_configs:
      - targets:
        - localhost:12345
    ```

3. Restart your local agent.

    ```sh
    sudo systemctl restart grafana-agent.service
    ```

Once you've done all of this, metrics should start flowing within a few minutes.

### Local Prometheus

This is essentially the same whether you're using it in agent mode or server mode.

1. Run the installer.

    ```sh
    curl https://raw.githubusercontent.com/dead-claudia/journald-exporter/main/install.sh | sudo bash -s -g prometheus
    ```

2. [Configure your Prometheus instance to scrape the service](https://prometheus.io/docs/prometheus/latest/configuration/configuration/#scrape_config) using the API key file generated from earlier as the password file.

    ```yaml
    # Add this to the `scrape_configs:` section of your Prometheus config
    - job_name: journald-exporter
      basic_auth:
        username: metrics
        password_file: /etc/prometheus-keys/journald-exporter.key
      static_configs:
      - targets:
        - localhost:12345
    ```

3. Restart your local Prometheus service.

    ```sh
    sudo systemctl restart prometheus.service
    ```

Once you've done all of this, metrics should start flowing within a few minutes.

### Remote Prometheus

This is essentially the same whether you're using it in agent mode or server mode.

1. Provision a certificate + private key pair. In the next step, `$cert` is the path to that certificate, and `$priv` is the path to its corresponding private key.

2. Run the installer.

    ```sh
    curl https://raw.githubusercontent.com/dead-claudia/journald-exporter/main/install.sh | sudo bash -s -c $cert -p $priv
    ```

3. Poke a hole in your firewall for port 12345 to allow inbound connections from it.

4. Copy `/etc/prometheus-keys/journald-exporter.key` file to your remote instance.

5. [Configure your Prometheus instance to scrape the service](https://prometheus.io/docs/prometheus/latest/configuration/configuration/#scrape_config) using the API key file generated from earlier as the password.

    ```yaml
    # Add this to the `scrape_configs:` section of your Prometheus config
    - job_name: journald-exporter
      basic_auth:
        username: metrics
        # You may need to change this to wherever you saved it on your remote
        # Prometheus instance.
        password_file: /etc/prometheus-keys/journald-exporter.key
      scheme: https
      static_configs:
      - targets:
        # Set this to whatever the host in question is.
        - your-host.example.com:12345
    ```

6. Restart your remote Prometheus service.

    ```sh
    sudo systemctl restart prometheus.service
    ```

Once you've done all of this, metrics should start flowing within a few minutes.

### Other local

It'll depend on what you're using.

1. Run the installer.

    ```sh
    curl https://raw.githubusercontent.com/dead-claudia/journald-exporter/main/install.sh | sudo bash
    ```

    If you can, set the key's owning group to the `$group` your scraper runs under by passing `-s -g $group` at the end. It'll save you a `chgrp $group /etc/prometheus-keys/journald-exporter.key` command and some time.

    ```sh
    # Alternative command
    curl https://raw.githubusercontent.com/dead-claudia/journald-exporter/main/install.sh | sudo bash -s -g $group
    ```

2. Configure your scraper to read from the service.

    - Scrape endpoint: `http://localhost:12345/metrics`
    - Authorization: Basic auth
    - Username: `metrics`
    - Password: the contents of `/etc/prometheus-keys/journald-exporter.key` (prefer to pass a file name instead of its contents if you can, for security reasons)

Once you've done all of this, metrics should start flowing within a few minutes.

### Other remote

It'll depend on what you're using.

1. Provision a certificate + private key pair. In the next step, `$cert` is the path to that certificate, and `$priv` is the path to its corresponding private key.

2. Run the installer.

    ```sh
    curl https://raw.githubusercontent.com/dead-claudia/journald-exporter/main/install.sh | sudo bash
    ```

    If your scraper is local, pass `-g SCRAPER_GROUP` if you can to set the owning key to the right group right off the bat. It'll save you a `chgrp SCRAPER_GROUP /etc/prometheus-keys/journald-exporter.key` command and some time.

3. Poke a hole in your firewall for port 12345 to allow inbound connections from it.

4. Configure your scraper to read from the service.

    - Scrape endpoint: `https://localhost:12345/metrics`
    - Authorization: Basic auth
    - Username: `metrics`
    - Password: the contents of the local `/etc/prometheus-keys/journald-exporter.key` (prefer to pass a file name instead of its contents if you can, for security reasons)

Once you've done all of this, metrics should start flowing within a few minutes.

### Manual

Of course, this can still be installed manually. It's tedious, as you can see, but you do get full control.

> This assumes a basic understanding of shell commands and shell variables.

1. If you plan to scrape from a remote machine, provision a certificate + private key pair. Place the certificate at `/etc/journald-exporter/cert.key` and the private key at `/etc/journald-exporter/priv.key`.

2. [Download the latest binary](https://github.com/dead-claudia/journald-exporter/releases) and install it somewhere in your `$PATH`. `/usr/sbin/journald-exporter` is recommended, as that's what the update script assumes.

    ```sh
    # Assumes `journald-exporter` is in the current directory and `/usr/sbin`
    # is in your default `$PATH`

    # Note: you have to use `mv` to replace the file to avoid "text file busy"
    # file access errors on update
    chmod 755 journald-exporter
    sudo cp journald-exporter journald-exporter.tmp
    sudo mv journald-exporter.tmp /usr/sbin/journald-exporter
    ```

3. Set up a `journald-exporter` system user with same-named group. No home directory is needed.

    ```sh
    sudo useradd --system --user-group journald-exporter
    ```

    This username is hard-coded into the exporter. If you *really* want a different name, you'll have to [update the source code and recompile it](CONTRIBUTING.md).

4. Place a systemd service unit file in `/etc/systemd/system/journald-exporter.service` with the following contents:

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
    ExecStart=/usr/sbin/journald-exporter --key-dir /etc/journald-exporter/keys --port 12345
    # Use this instead if you have an HTTPS certificate provisioned
    #ExecStart=/usr/sbin/journald-exporter --key-dir /etc/journald-exporter/keys --port 12345 --certificate /etc/journald-exporter/cert.key --private-key /etc/journald-exporter/priv.key
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
    ```

    If you want, or need, to change the key directory or port number, this is where you'll need to configure it.

5. Make the `/etc/journald-exporter/keys` directory mentioned above, with an owner of root and permissions of 755 (user has all perms, everyone else has only read/execute). This is where you'll put your API keys.

    ```sh
    sudo mkdir --mode=755 /etc/journald-exporter/keys
    ```

6. Create an API key (must be pure hexadecimal but may be surrounded by whitespace in the file) and copy it to `/etc/journald-exporter/keys` with an owner of root and permissions of 600 (root can read and write, nobody else can access).

    ```sh
    key_name="$(date -uIseconds).key"
    sudo openssl rand -hex -out "$key_name" 32
    sudo chmod 600 "$key_name"
    sudo mv "$key_name" "/etc/journald-exporter/keys/$key_name"
    ```

    You'll also want to copy this key - you'll need it later. You can use the following (using `$name` from above) to set up the key for a local Prometheus instance:

    ```sh
    sudo mkdir -p /etc/prometheus-keys
    sudo cp "/etc/journald-exporter/keys/$name" /etc/prometheus-keys/journald-exporter.key
    sudo chgrp $prometheus_user /etc/prometheus-keys/journald-exporter.key
    sudo chmod 640 /etc/prometheus-keys/journald-exporter.key
    ```

7. Start the service.

    ```sh
    sudo systemctl start journald-exporter.service
    ```

8. If you plan to scrape from a remote machine, poke a hole in your firewall to allow inbound connections from port 12345.

9. Configure your scraper to read from the service.

    - Scrape endpoint: `http://localhost:12345/metrics` or `https://your-host.example.com:12345/metrics` as applicable
    - Authorization: Basic auth
    - Username: `metrics`
    - Password: the contents of the local `/etc/prometheus-keys/journald-exporter.key` (prefer to pass a file name instead of its contents if you can, for security reasons)

Once you've done all of this, metrics should start flowing within a few minutes.

To update, just reinstall the binary per the first installation step and restart the service via `systemctl restart journald-exporter.service`.

## Update

### Updating the binary

Just run this command. Doesn't matter what you're using to scrape it with.

```sh
curl https://raw.githubusercontent.com/dead-claudia/journald-exporter/main/update.sh | sudo bash
```

You can also download that script and just run it locally if you prefer.

It's also easy to do manually:

1. [Download the latest release.](https://github.com/dead-claudia/journald-exporter/releases)
2. Take the file, make a copy on the same partition as the root binary if needed, and rename it to `/usr/sbin/journald-exporter`. (This avoids permissions errors, since you can't directly write to a running executable.)
3. Restart the service: `sudo systemctl restart journald-exporter.service`

### Updating certificates

You'll need to have the existing certificates handy. Should take very little time to swap them out.

1. Replace the certificate at `/etc/journald-exporter/cert.key` with the new certificate.
2. Replace the private key at `/etc/journald-exporter/priv.key` with the new private key.
3. Restart the service: `sudo systemctl restart journald-exporter.service`
