FROM rust:1.75 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release --bin redis_server

FROM debian:bookworm-slim
COPY --from=builder /usr/src/app/target/release/redis_server /usr/local/bin/
EXPOSE 6379
CMD ["redis_server"]