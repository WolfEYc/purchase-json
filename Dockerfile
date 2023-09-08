FROM rust:alpine as builder

WORKDIR /app/src
RUN USER=root

RUN apk add pkgconfig openssl-dev libc-dev
COPY ./ ./
RUN cargo build --release

FROM alpine:latest
WORKDIR /app
RUN apk update \
    && apk add openssl ca-certificates

EXPOSE 8080

COPY --from=builder /app/src/target/release/purchase-json /app/purchase-json

CMD ["/app/purchase-json"]