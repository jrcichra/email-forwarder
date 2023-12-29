FROM rust:1.75.0-bullseye as builder
WORKDIR /app
# https://users.rust-lang.org/t/cargo-uses-too-much-memory-being-run-in-qemu/76531
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
RUN cargo init
COPY Cargo.toml Cargo.lock /app/
RUN cargo build --release
COPY src/main.rs /app/src/main.rs
RUN touch src/main.rs && cargo build --release

FROM debian:bullseye-20231218-slim
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/email-forwarder /email-forwarder
ENTRYPOINT ["/email-forwarder"]
