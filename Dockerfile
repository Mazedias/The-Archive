FROM rust:1.81 as builder

WORKDIR /usr/src/the-archive

COPY . .

RUN cargo build --release

FROM debian:buster-slim

RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/the-archive/target/release/the-archive .

COPY --from=builder /usr/src/the-archive/static ./static
COPY --from=builder /usr/src/the-archive/templates ./templates
COPY --from=builder /usr/src/the-archive/content ./content

EXPOSE 8000

CMD ["./the-archive"]           