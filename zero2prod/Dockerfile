# We use the latest stable release as the base image
FROM rust:1.94 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

