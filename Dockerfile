FROM rust:1.31

RUN mkdir -p /usr/local/src/drain
WORKDIR /usr/local/src/drain

COPY . /usr/local/src/drain

RUN cargo test
