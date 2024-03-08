# syntax=docker/dockerfile:1
FROM rust:1.75-slim as build
WORKDIR /
COPY . .
RUN apt-get update
RUN apt-get install -y pkg-config curl
RUN apt-get install -y libssl-dev openssl
RUN ["cargo", "build", "--release"]

FROM python:3.12-slim 
COPY --from=build /target/release/r6rs /r6rs
COPY --from=build /assets /assets
COPY --from=build /sherlock /sherlock
RUN pip install -r "/sherlock/sherlock/requirements.txt"
VOLUME /data
CMD ["/r6rs"]
EXPOSE 3000