# Hopper ![License](https://img.shields.io/github/license/BRA1L0R/hopper-rs?style=flat-square)

<img src="./.github/hopper.webp" align="right" width="180">

**Hopper** is a lightweight reverse proxy for minecraft. It allows you to connect multiple servers under the same IP and port, with additional functionalities, just like **Nginx**. It is built with **Rust 🦀** to ensure maximum performance and efficiency.

Hopper works starting from version **1.7** up to the **latest** version of Minecraft.

## Configuration

Example `Config.toml`:

```toml
# the address hopper will listen on
listen = "0.0.0.0:25565"

# general routing configuration
[routing]
default = "127.0.0.1:12345" # optional

# list of servers fronted by hopper
[routing.routes]
# simple reverse proxy
"mc.gaming.tk" = "127.0.0.1:25008"

# this will load balance between the two servers
"other.gaming.tk" = ["127.0.0.1:25009", "10.1.0.1:25123"]
```

# Load balancer

TODO: write explaination, check config example for now

## How to run

- TODO: cargo build explaination
- TODO: Doker image
- TODO: Github release
