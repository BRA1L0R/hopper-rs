# Build stage
FROM rust:latest as build

RUN USER=root cargo new --bin hopper
WORKDIR /hopper

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# cache deps
RUN cargo build --release 

RUN rm ./src/*.rs
COPY ./src ./src

RUN rm ./target/release/deps/hopper*
RUN cargo build --release

# final runnable image
FROM debian:bookworm-slim
COPY --from=build /hopper/target/release/hopper .

RUN chmod a+x ./hopper
CMD ["./hopper"]