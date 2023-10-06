FROM rustlang/rust:nightly as builder

WORKDIR /phixiv

COPY . .

RUN cargo build --release

FROM debian:bullseye
COPY --from=builder /phixiv/target/release/phixiv .

CMD [ "./phixiv" ]