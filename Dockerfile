FROM rust:1.68.0 as builder

ARG FEATURES=bot_filtering

RUN mkdir phixiv
WORKDIR /phixiv

COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

RUN cargo build --release --features ${FEATURES}

FROM debian:buster-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /phixiv/target/release/phixiv .

CMD [ "./phixiv" ]