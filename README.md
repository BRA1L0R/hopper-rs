# Hopper ![License](https://img.shields.io/github/license/BRA1L0R/hopper-rs?style=flat-square)

<img src="./.github/hopper.webp" align="right" width="180">

**Hopper** is a lightweight reverse proxy for minecraft. It allows you to connect multiple servers under the same IP and port, with additional functionalities, just like **Nginx**. It is built with **Rust 🦀** to ensure maximum performance and efficiency.

Hopper works starting from version **1.7** up to the **latest** version of Minecraft.

NOTE: this proxy is still heavily under development, and a lot of new features are coming really soon!

**FEATURES:**

- [x] Load balancing
- [x] [IP Forwarding](#ip-forwarding)
- [x] [HAProxy v2 (PROXY protocol)](#proxy-protocol-aka-haproxy-v2)
- [x] [RealIP](#realip) 2.4 support
- [x] [Logging metrics](#logging-metrics-with-influxdb) with InfluxDB
- [x] Forge support
- [ ] Webhook callbacks for events
- [ ] Plugin system for Docker and hosting provider integrations

## Configuration

Example `Config.toml`:

```toml
# the address hopper will listen on
listen = "0.0.0.0:25565"

# metrics configuration
# [metrics]
# type = "influx"
# ...
#
# follow the section below for more information about
# gathering metrics with hopper

# general routing configuration
[routing]
default = { ip = "127.0.0.1:12345" } # optional
# default = { ip = ["127.0.0.1:12001", "127.0.0.1:12002"] } # load balanced

# list of servers fronted by hopper
[routing.routes]
# simple reverse proxy
"mc.gaming.tk" = { ip = "docker_hostname:25008" } # hostnames are supported too!

# bungeecord's ip forwarding feature enabled
"mc.server.com" = { ip-forwarding = "bungeecord", ip = "127.0.0.1:25123" }

# RealIP ip forwarding feature enabled
"mc.withrealip.com" = { ip-forwarding = "realip", ip = "127.0.0.1:26161" }

# this will load balance between the two servers
"other.gaming.tk" = { ip = ["127.0.0.1:25009", "10.1.0.1:25123"] }
```

### IP Forwarding

Without IP Forwarding, when servers receive connections from this reverse proxy they won't see the original client's ip address. This may lead to problems with sessions with plugins such as AuthMe. Hopper implements both the legacy BungeeCord protocol and the more versatile RealIP one.

#### Bungeecord

You must enable bungeecord ip-forwarding inside of `spigot.yml` just like you would using bungeecord. [Click here](https://shockbyte.com/billing/knowledgebase/38/IP-Forwarding-in-BungeeCord.html) to learn more.

You can enable ip forwarding **per-server** on hopper with the "ip-forwarding" directive:

```toml
# You can either do it this way
[routing.routes]
"your.hostname.com" = { ip-forwarding = "bungeecord", ip = "<your server ip>" }

# or this way
[routing.routes."your.hostname.com"]
ip-forwarding = "bungeecord" # available options are: bungeecord, none. Defaults to none
ip = "<your server ip>"
```

#### RealIP

Hopper supports up to RealIP v2.4 (private/public key authentication has been implemented for versions after that, which only works with TCPShield).

⚠️ Note: RealIP v2.4 was built using older dependencies, hence support for newer minecraft versions may be lacking.

You must whitelist Hopper's ip address (or network) by adding a line inside of `plugins/TCPShield/ip-whitelist/tcpshield-ips.list`.

Finally, you must enable RealIP support in your `Config.toml`:

```toml
[routing.routes]
"your.hostname.com" = { ip-forwarding = "realip", ip = "<your server ip>" }
```

#### PROXY Protocol a.k.a HAProxy V2

Support for [PROXY Protocol](https://www.haproxy.org/download/1.8/doc/proxy-protocol.txt) is available by setting `ip-forwarding` to `proxy_protocol`. Only the 2nd version of
the protocol (the one supported by Bungeecord out of the box) is implemented in Hopper.

Example configuration:

```toml
[routing.routes]
"my.bungee.hostname.com" = { ip-forwarding = "proxy_protocol", ip = "<your server ip>" }
```

### Logging metrics with InfluxDB

Hopper supports **cheap** (resource-wise), easily configurable data gathering through the help of an external database like InfluxDB (although other databases will be supported in the future, I still recommend InfluxDB whose query language is very easy and versatile).

You must first configure an InfluxDB instance and get a **token** with writing privilege before moving along with this section.

Add and modify this configuration section in `Config.toml` according to your setup:

```toml
[metrics] # top-level section
type = "influxdb"
# hostname = "my-hostname" # OPTIONAL, defaults to system hostname
url = "<http/https>://<influxdb-host-or-ip>:<port>/"
organization = "<Your organization>"
bucket = "<Your data bucket>"
token = "<Your access token>"
```

Hopper will start logging every **5 seconds** according to this data format:

**Measurement "traffic":**
| Field | Type | Description |
| ----- | ---- | ----------- |
| hostname | Tag | system (or custom if specified) hostname generating this metric |
| dest_hostname | Tag | the hostname clients connected corresponding to these metrics |
| clientbound_bandwidth | Value (int) | the traffic this host generated server=>client |
| serverbound_bandwidth | Value (int) | same as above, but client=>server |
| open_connections | Value(int) | connections opened in the moment of the measurement |
| total_game | Value(int) | people who attemped or succeded joining this server |
| total_ping | Value(int) | people who pinged this server |

_NOTE: As counters reset through restarts, data manipulation using the influx query language allows you to aggregate rows and get persistent results._

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
version: "3"

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

## Changing the verbosity level

If you think something is off with your instance and want to enable debug logging, or you just want to reduce the default talkativeness of hopper you must choose your desired level of verbosity through the `RUST_LOG` environment variable.

| Level | Description                                                                     |
| ----- | ------------------------------------------------------------------------------- |
| off   | No console output at all                                                        |
| error | Only output important errors such as an unreachable backend server              |
| info  | Informative data such as incoming connections and the current listening address |
| debug | More descriptive errors (includes failed handshakes and bad packet data)        |

_Default: `info`_

Example:

```sh
RUST_LOG="debug" ./hopper
```

TODO: running information with systemd configuration example
