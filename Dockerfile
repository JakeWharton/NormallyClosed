# Cross-compile the app for musl to create a statically-linked binary for alpine.
FROM rust:1.50 AS rust
RUN rustup target add armv7-unknown-linux-musleabihf
RUN rustup install nightly  # for rustfmt
RUN rustup component add clippy
RUN rustup component add rustfmt --toolchain nightly
RUN apt-get update && apt-get -y install binutils-arm-linux-gnueabihf
WORKDIR /app
COPY .cargo ./.cargo
COPY Cargo.toml Cargo.lock .rustfmt.toml ./
COPY src ./src
RUN cargo build --release
RUN cargo clippy --release
RUN cargo +nightly fmt -- --check


FROM golang:alpine AS shell
RUN apk add --no-cache shellcheck
ENV GO111MODULE=on
RUN go get mvdan.cc/sh/v3/cmd/shfmt
WORKDIR /overlay
COPY root/ ./
COPY .editorconfig /
RUN find . -type f | xargs shellcheck -e SC1008
RUN shfmt -d .


FROM oznu/s6-alpine:3.13-armhf
ENV \
    # Fail if cont-init scripts exit with non-zero code.
    S6_BEHAVIOUR_IF_STAGE2_FAILS=2 \
    # Show full backtraces for crashes.
    RUST_BACKTRACE=full \
    GARAGE_PIE_ARGS=""
COPY root/ /
WORKDIR /app
COPY --from=rust /app/target/armv7-unknown-linux-musleabihf/release/garage_pie ./
