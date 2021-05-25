# Cross-compile the app for musl to create a statically-linked binary for alpine.
FROM rust:1.52.1 AS rust
RUN rustup target add armv7-unknown-linux-musleabihf
RUN apt-get update && apt-get -y install binutils-arm-linux-gnueabihf
WORKDIR /app
COPY .cargo ./.cargo
COPY Cargo.toml Cargo.lock .rustfmt.toml ./
COPY src ./src
RUN cargo build --release --target armv7-unknown-linux-musleabihf


FROM alpine:3.12
ENV \
    # Show full backtraces for crashes.
    RUST_BACKTRACE=full
RUN apk add --no-cache \
      tini \
    && rm -rf /var/cache/* \
    && mkdir /var/cache/apk
WORKDIR /app
COPY --from=rust /app/target/armv7-unknown-linux-musleabihf/release/normally-closed ./

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/app/normally-closed", "--http-port", "80", "/config/config.toml"]

EXPOSE 80
HEALTHCHECK --interval=1m --timeout=3s \
  CMD wget --no-verbose --tries=1 --spider http://localhost/ || exit 1
