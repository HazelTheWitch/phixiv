FROM rust:1.73 as builder

WORKDIR /usr/src/phixiv
COPY . .
RUN cargo install --path .

FROM debian:bullseye

RUN apt-get update && apt-get install -y pkg-config libssl-dev

COPY --from=builder /usr/local/cargo/bin/phixiv /usr/local/bin/phixiv

CMD [ "phixiv" ]