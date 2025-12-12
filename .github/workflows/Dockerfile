ARG RUST_VERSION=1.90.0
FROM rust:${RUST_VERSION} as m
RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/debian.sources && apt update -y && apt upgrade -y
RUN arch && apt install -y musl-tools && rustup target add $(arch)-unknown-linux-musl
RUN git clone https://github.com/yeslogic/allsorts && cd allsorts && git checkout -b 0.16.1
#FROM registry.gitlab.com/rust_musl_docker/image:stable-latest as m

COPY . /app
RUN cp -r allsorts /app && cd /app/allsorts && sed -i "31 s@base@pub base@;36 s@base@pub base@" src/binary/read.rs 
WORKDIR /app

RUN cargo clean && cargo build --release \
    --config source.crates-io.replace-with=\"mirror\" \
    --config source.mirror.registry=\"sparse+https://mirrors.ustc.edu.cn/crates.io-index/\" -vv \
    --target=$(arch)-unknown-linux-musl \
    && mkdir out \
    && mv target/$(arch)-unknown-linux-musl/release/fontview ./out/fontview


FROM scratch
#FROM alpine
COPY --from=m /app/out/ /app/