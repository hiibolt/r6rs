# syntax=docker/dockerfile:1
FROM rust:1.75-slim as build
WORKDIR /
COPY . .
RUN apt-get update
RUN apt-get install -y pkg-config curl
RUN apt-get install -y libssl-dev openssl
RUN ["cargo", "build", "--release"]

VOLUME /data
CMD ["/target/release/r6rs"]
EXPOSE 3000