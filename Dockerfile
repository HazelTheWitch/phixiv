FROM rustlang/rust:nightly as builder

ARG FEATURES=bot_filtering

RUN mkdir phixiv
WORKDIR /phixiv

COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./templates ./templates

RUN cargo build --release --features ${FEATURES}

FROM debian:buster-slim
COPY --from=builder /phixiv/target/release/phixiv .

CMD [ "./phixiv" ]