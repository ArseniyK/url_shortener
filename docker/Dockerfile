FROM rust:1.49.0-buster as builder

ENV DOCKERIZE_VERSION=v0.6.1 \
    PKG_CONFIG_ALLOW_CROSS=1
# muslc is required in order to build the rust image.
RUN apt-get update \
    && apt-get -y install \
    ca-certificates \
    cmake \
    musl-tools \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && wget "https://github.com/jwilder/dockerize/releases/download/${DOCKERIZE_VERSION}/dockerize-alpine-linux-amd64-${DOCKERIZE_VERSION}.tar.gz" \
    && tar -C /usr/local/bin -xzvf "dockerize-alpine-linux-amd64-${DOCKERIZE_VERSION}.tar.gz" \
    && rm "dockerize-alpine-linux-amd64-${DOCKERIZE_VERSION}.tar.gz"

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /code
RUN USER=root cargo new url_shortener

COPY Cargo.toml Cargo.lock /code/url_shortener/

WORKDIR /code/url_shortener/

RUN cargo fetch
RUN cargo install cargo-watch
COPY . /code/url_shortener
RUN cargo build --target x86_64-unknown-linux-musl --release
CMD "cargo watch -x run"

FROM alpine:3.8 as production_build

RUN apk --no-cache add ca-certificates
COPY --from=builder /code/url_shortener/templates/ /url_shortener/templates
COPY --from=builder /code/url_shortener/static /url_shortener/static
COPY --from=builder /code/url_shortener/target/x86_64-unknown-linux-musl/release/url_shortener /url_shortener/url_shortener
WORKDIR /url_shortener
