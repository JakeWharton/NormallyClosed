# Cross-compile the app for musl to create a statically-linked binary for alpine.
FROM rust:1.52.1 AS rust
RUN rustup target add armv7-unknown-linux-musleabihf
RUN apt-get update && apt-get -y install binutils-arm-linux-gnueabihf
WORKDIR /app
COPY .cargo ./.cargo
COPY Cargo.toml Cargo.lock .rustfmt.toml ./
COPY src ./src
RUN cargo build --release --target armv7-unknown-linux-musleabihf


FROM oznu/s6-alpine:3.13-armhf
ENV \
    # Fail if cont-init scripts exit with non-zero code.
    S6_BEHAVIOUR_IF_STAGE2_FAILS=2 \
    # Show full backtraces for crashes.
    RUST_BACKTRACE=full
COPY root/ /
WORKDIR /app
COPY --from=rust /app/target/armv7-unknown-linux-musleabihf/release/normally-closed ./

EXPOSE 80
HEALTHCHECK --interval=1m --timeout=3s \
  CMD wget --no-verbose --tries=1 --spider http://localhost/ || exit 1
