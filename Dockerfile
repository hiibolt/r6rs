# syntax=docker/dockerfile:1
FROM rust:1.75-slim as build
WORKDIR /
COPY . .
RUN apt-get update
RUN apt-get install -y pkg-config curl
RUN apt-get install -y libssl-dev openssl
RUN ["cargo", "build", "--release"]

FROM python:3.12-slim 

# Import needed data
COPY --from=build /target/release/r6rs /r6rs
COPY --from=build /assets /assets
VOLUME /data

# Add Tini
ENV TINI_VERSION v0.19.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini /tini
RUN chmod +x /tini
ENTRYPOINT ["/tini", "--"]

# Install Python dependencies for `plotpy`
RUN pip install numpy
RUN pip install matplotlib

CMD ["/r6rs"]
EXPOSE 3000