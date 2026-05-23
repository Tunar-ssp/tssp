# Installation Guide

This guide describes how to install and run the TSSP (Transfer and Storage System) daemon and CLI on Linux environments, such as Ubuntu or an Orange Pi.

## Prerequisites

- **Rust toolchain** (version 1.93 or later). Install via [rustup](https://rustup.rs/):
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **SQLite3 development libraries**:
  ```sh
  sudo apt update
  sudo apt install libsqlite3-dev
  ```

## Building from Source

1. Clone the repository:
   ```sh
   git clone https://github.com/Tunar-ssp/tssp.git
   cd tssp
   ```

2. Build the entire workspace (daemon and CLI) in release mode:
   ```sh
   cargo build --release --workspace
   ```

3. The compiled binaries will be available in `target/release/`:
   - `target/release/tsspd` (Daemon)
   - `target/release/tssp` (CLI)

## Installing the Daemon (`tsspd`)

For production or continuous usage, it is recommended to run `tsspd` as a systemd service.

1. **Create a dedicated user and data directory**:
   ```sh
   sudo useradd -r -m -d /var/lib/tssp -s /usr/sbin/nologin tssp
   sudo mkdir -p /var/lib/tssp/data
   sudo chown -R tssp:tssp /var/lib/tssp
   ```

2. **Copy the binary**:
   ```sh
   sudo cp target/release/tsspd /usr/local/bin/
   sudo chmod +x /usr/local/bin/tsspd
   ```

3. **Create a systemd service file** (`/etc/systemd/system/tsspd.service`):
   ```ini
   [Unit]
   Description=TSSP Backend Daemon
   After=network.target

   [Service]
   Type=simple
   User=tssp
   Group=tssp
   ExecStart=/usr/local/bin/tsspd --bind 0.0.0.0 --port 8421 --data-dir /var/lib/tssp/data
   Restart=on-failure
   Environment="RUST_LOG=info"

   [Install]
   WantedBy=multi-user.target
   ```

4. **Enable and start the service**:
   ```sh
   sudo systemctl daemon-reload
   sudo systemctl enable tsspd
   sudo systemctl start tsspd
   sudo systemctl status tsspd
   ```

## Installing the CLI (`tssp`)

The CLI is intended to be used by individual users on their local machines.

1. **Copy the binary to your local path**:
   ```sh
   sudo cp target/release/tssp /usr/local/bin/
   sudo chmod +x /usr/local/bin/tssp
   ```

2. **Generate shell completions** (e.g., for Bash):
   ```sh
   tssp completions bash | sudo tee /etc/bash_completion.d/tssp > /dev/null
   source /etc/bash_completion.d/tssp
   ```

3. **Verify installation**:
   ```sh
   tssp status
   ```

## Configuration

By default, the CLI assumes the daemon is running on `127.0.0.1:8421`. 
To connect to a daemon on a different host (e.g., your Orange Pi on the local network), you can use the connection arguments:

```sh
tssp --host 192.168.1.100 --port 8421 status
```

You can also set environment variables `TSSP_HOST` and `TSSP_PORT` in your `.bashrc` or `.zshrc` to avoid passing them manually:
```sh
export TSSP_HOST=192.168.1.100
export TSSP_PORT=8421
```
