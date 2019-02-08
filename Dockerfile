FROM rust:1.32

WORKDIR /usr/src/iron-spider
COPY . .


RUN apt-get update && apt-get -y install libsodium-dev pkg-config
RUN cargo install --path .

CMD ["iron-spider"]
