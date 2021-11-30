FROM rust:alpine AS builder

WORKDIR /work

RUN apk update && apk add musl-dev protoc

COPY jcss ./jcss

COPY jcss-cli ./jcss-cli

COPY jcss-web ./jcss-web

COPY Cargo.toml Cargo.lock ./

RUN cargo build --bin jcss-web --release

FROM alpine:latest

WORKDIR /work

COPY --from=builder ./work/target/release/jcss-web ./

COPY ./model.onnx ./

EXPOSE 8000

ENV APP_BIND=0.0.0.0:8000

ENV APP_MODEL=model.onnx

CMD ["./jcss-web"]
