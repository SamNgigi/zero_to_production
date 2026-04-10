# We use the latest stable release as the base image
FROM rust:1.94 AS builder
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release
ENTRYPOINT [ "./target/release/zero2prod" ]
