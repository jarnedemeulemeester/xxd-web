FROM alpine:3.11

RUN apk add --no-cache xxd

WORKDIR /app

COPY target/x86_64-unknown-linux-musl/release/xxd-web .

COPY static static

EXPOSE 8000

CMD ["./xxd-web"]