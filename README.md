# Hopper ![License](https://img.shields.io/github/license/BRA1L0R/hopper-rs?style=flat-square)

<img src="./.github/hopper.webp" align="right" width="180">

**Hopper** is a lightweight reverse proxy for minecraft. It allows you to connect multiple servers under the same IP and port, with additional functionalities, just like **Nginx**. It is built with **Rust 🦀** to ensure maximum performance and efficiency.

Hopper works starting from version **1.7** up to the **latest** version of Minecraft.

NOTE: this proxy is still heavily under development, and a lot of [new features](#upcoming-features) are coming really soon!

**FEATURES:**
- [x] Load balancing
- [x] [IP Forwarding](#ip-forwarding)
- [ ] Webhook callbacks for events
- [ ] Logging metrics on InfluxDB
- [ ] Rest api for metrics and operation
- [ ] Plugin system for Docker and hosting provider integrations

## Configuration

Example `Config.toml`:

```toml
# the address hopper will listen on
listen = "0.0.0.0:25565"

# general routing configuration
[routing]
default = { ip = "127.0.0.1:12345" } # optional
# default = { ip = ["127.0.0.1:12001", "127.0.0.1:12002"] } # load balanced

# list of servers fronted by hopper
[routing.routes]
# simple reverse proxy
"mc.gaming.tk" = { ip = "127.0.0.1:25008" }

# bungeecord's ip forwarding feature enabled
"mc.server.com" = { ip-forwarding = true, ip "127.0.0.1:25123" }

# this will load balance between the two servers
"other.gaming.tk" = ["127.0.0.1:25009", "10.1.0.1:25123"]
```

### IP Forwarding

Without IP Forwarding, when servers receive connections from this reverse proxy they won't see the original client's ip address. This may lead to problems with sessions with plugins such as Authme. Hopper implements the same "protocol" BungeeCord uses (old but very compatible with all Minecraft versions).

⚠️ Note: you will also need to enable the bungeecord directive in your server's configuration files. [Click here](https://shockbyte.com/billing/knowledgebase/38/IP-Forwarding-in-BungeeCord.html) to learn more.

You can enable ip forwarding per-server on hopper with the "ip-forwarding" directive like this:
```toml
[routing.routes."your.hostname.com"]
ip-forwarding = true # defaults to false
ip = "<your server ip>"
```

## How to run

There are two ways to run hopper:
- Using the [docker image](#docker-)
- Using the [binaries](#binary-)

### Docker ![GitHub Workflow Status](https://img.shields.io/github/workflow/status/bra1l0r/hopper-rs/Docker%20build%20and%20registry%20push?label=Container%20Build&style=flat-square)

- Pull the latest image from the GitHub registry:
```sh
docker pull ghcr.io/bra1l0r/hopper-rs
```

- Create a `Config.toml` (NOTE: the port you will specify must match the exposed port below)
- Run it using docker:
```sh
docker run -d -p 25565:25565 -v /home/user/path-to/Config.toml:/Config.toml ghcr.io/bra1l0r/hopper-rs
```

#### Using docker-compose **(recommended)**:
```yaml
# new versions of compose don't require this anymore
version: '3'

services:
  hopper:
    image: ghcr.io/bra1l0r/hopper-rs
    ports:
      - 25565:25565
    volumes:
      - ./Config.toml:/Config.toml
```

### Binary ![GitHub Workflow Status](https://img.shields.io/github/workflow/status/bra1l0r/hopper-rs/Build%20and%20release%20on%20github?label=Artifact%20Release&style=flat-square)

You can either download the [latest release](https://github.com/BRA1L0R/hopper-rs/releases) **(recommended)** or follow the steps below to build your own binary:

- Download and install the latest version of the rustc toolchain
- Clone and build the repo:
```sh
# Clone the repo into hopper-rs and enter the directory
git clone https://github.com/BRA1L0R/hopper-rs
cd hopper-rs/

# Build the project with the release profile
cargo build --release
``` 
- The runnable binary will now be available at `target/release/hopper`

TODO: running information with systemd configuration example
