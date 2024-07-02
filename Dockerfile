FROM gcr.io/distroless/base-debian12:debug-nonroot as rename
WORKDIR /app
COPY target/aarch64-unknown-linux-gnu/release/email-forwarder email-forwarder-arm64
COPY target/x86_64-unknown-linux-gnu/release/email-forwarder email-forwarder-amd64

FROM debian:12.6-slim
ARG TARGETARCH
COPY --from=rename /app/email-forwarder-$TARGETARCH /app/email-forwarder
ENTRYPOINT [ "/app/email-forwarder" ]
