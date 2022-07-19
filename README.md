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
# default = ["127.0.0.1:12001", "127.0.0.1:12002"] # load balanced

# list of servers fronted by hopper
[routing.routes]
# simple reverse proxy
"mc.gaming.tk" = "127.0.0.1:25008"

# this will load balance between the two servers
"other.gaming.tk" = ["127.0.0.1:25009", "10.1.0.1:25123"]
```

## How to run

There are two ways to run hopper:
- Using the [docker image](#docker)
- Using the [binaries](#binaries)

### Docker

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

### Binary

You can either build the latest commit or download the [latest release](https://github.com/BRA1L0R/hopper-rs/releases)

#### BIY (Build-It-Yourself):

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

#### Running the binary

TODO: running

## TODO

- [x] write `Dockerfile` and `docker-compose.yml`
- [x] github ci / cd with code check, build and release
- [x] run and build documentation in readme
- [x] restructure router trait
- [ ] add support for influxdb metrics (or similar)
- [ ] rest api for metrics and operation
- [ ] webhook callbacks for events
- [ ] benchmark comparison with similar programs
