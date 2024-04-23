FROM gcr.io/distroless/base-debian12:debug-nonroot as rename
WORKDIR /app
COPY target/aarch64-unknown-linux-musl/release/email-forwarder email-forwarder-arm64
COPY target/x86_64-unknown-linux-musl/release/email-forwarder email-forwarder-amd64

FROM gcr.io/distroless/static-debian12:nonroot
ARG TARGETARCH
COPY --from=rename /app/email-forwarder-$TARGETARCH /app/email-forwarder
ENTRYPOINT [ "/app/email-forwarder" ]
