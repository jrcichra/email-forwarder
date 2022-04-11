FROM rust:1.60-slim-bullseye as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bullseye-20220328-slim
COPY --from=builder /usr/src/app/target/release/email-fanout /email-fanout
ENTRYPOINT ["/email-fanout"]
