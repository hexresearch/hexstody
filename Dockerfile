FROM rust:1 as builder
RUN apt-get update && apt-get install -y clang postgresql sudo
COPY . .
RUN ./docker/setup-build-postgresql.sh

FROM debian:bullseye-slim
COPY --from=builder target/release/hexstody-hot /hexstody-hot
VOLUME /data
WORKDIR /data
RUN apt update && apt install -y libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY ./docker/wait-for-it.sh /wait-for-it.sh
