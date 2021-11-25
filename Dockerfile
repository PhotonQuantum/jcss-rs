FROM rust:slim-bullseye AS builder

WORKDIR /work

RUN apt-get -y update

RUN apt-get -y install pkg-config libssl-dev ca-certificates

COPY jcss ./jcss

COPY jcss-cli ./jcss-cli

COPY jcss-web ./jcss-web

COPY Cargo.toml Cargo.lock ./

RUN cargo build --bin jcss-web --release

FROM debian:bullseye-slim

WORKDIR /work

RUN apt-get -y update

RUN apt-get -y install ca-certificates

COPY --from=builder ./work/target/release/jcss-web ./

COPY ./model.onnx ./

EXPOSE 8000

ENV APP_BIND=0.0.0.0:8000

ENV APP_MODEL=model.onnx

CMD ["./jcss-web"]
