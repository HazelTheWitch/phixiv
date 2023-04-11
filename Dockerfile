FROM rustlang/rust:nightly as builder

RUN mkdir phixiv
WORKDIR /phixiv

COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./templates ./templates

RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /phixiv/target/release/phixiv .

CMD [ "./phixiv" ]