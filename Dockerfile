FROM rustlang/rust:nightly as builder

RUN cargo new --bin phixiv

WORKDIR /phixiv

COPY ./Cargo.toml ./Cargo.toml

RUN cargo check --release

RUN rm -rf ./src

COPY ./src ./src
COPY ./templates ./templates

RUN cargo build --release

FROM debian:bullseye

RUN apt-get update && apt-get install -y pkg-config libssl-dev

COPY --from=builder /phixiv/target/release/phixiv .

CMD [ "./phixiv" ]