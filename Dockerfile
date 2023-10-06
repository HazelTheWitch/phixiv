FROM rust:latest as builder

WORKDIR /usr/src/phixiv
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y openssl

COPY --from=builder /usr/local/cargo/bin/phixiv /usr/local/bin/phixiv

CMD [ "phixiv" ]