FROM rust:1.67 as builder
WORKDIR /usr/src/phixiv
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/phixiv /usr/local/bin/phixiv
CMD ["phixiv"]