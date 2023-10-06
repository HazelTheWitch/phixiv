FROM rust:1.73

WORKDIR /phixiv
COPY . .

RUN cargo install --path .

CMD ["phixiv"]