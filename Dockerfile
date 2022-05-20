FROM rust:latest as builder

WORKDIR /app/

COPY . .

RUN cargo build --release

FROM debian:buster-slim as server

COPY --from=builder /app/target/release/server /usr/local/bin/server

EXPOSE 8080

CMD ["server"]

# docker build -t public-chat:latest .
# docker run -d -p 8080:8080 public-chat:latest
