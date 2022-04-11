FROM rust:1.60-slim-bullseye as builder
WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release

FROM debian:bullseye-20220328-slim
RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/email-fanout /email-fanout
ENTRYPOINT ["/email-fanout"]
