FROM rust:1.32 as build

RUN USER=root cargo new --bin iron-spider
WORKDIR /iron-spider
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN apt-get update && apt-get -y install libsodium-dev pkg-config

#build dependencies an cache
RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

#to rebuild any changes ive made
RUN rm ./target/release/deps/iron_spider*

RUN cargo build --release

#FROM alpine:latest
FROM debian:stretch-slim
ENV ON_CLOUD=true
COPY --from=build /iron-spider/target/release/iron-spider .

#RUN apk update && apk add libssl1.1 libsodium ca-certificates bash
RUN apt-get update && apt-get -y install libsodium18 libssl1.1 ca-certificates

CMD ["./iron-spider"]
