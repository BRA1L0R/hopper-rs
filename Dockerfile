# Build stage
FROM rust:latest as build

RUN USER=root cargo new --bin hopper
WORKDIR /hopper

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# proc macros
COPY ./macros ./macros

# cache deps
RUN cargo build --release 

RUN rm ./src/*.rs
COPY ./src ./src

RUN rm ./target/release/deps/hopper*
RUN cargo build --release

# final runnable image
FROM alpine:latest
COPY --from=build /hopper/target/release/hopper .

# add deps
RUN apk add gcompat
RUN apk add libgcc

RUN chmod a+x ./hopper
CMD ["./hopper"]