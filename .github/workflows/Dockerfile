ARG RUST_VERSION=1.90.0
FROM rust:${RUST_VERSION} as m
RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/debian.sources && apt update -y && apt upgrade -y
RUN arch && apt install -y musl-tools && rustup target add $(arch)-unknown-linux-musl
#FROM registry.gitlab.com/rust_musl_docker/image:stable-latest as m
WORKDIR /app
COPY . /app
RUN cargo clean &&  cargo build --release --config source.crates-io.replace-with=\"mirror\" \
    --config source.mirror.registry=\"sparse+https://mirrors.ustc.edu.cn/crates.io-index/\" \
    && cargo build --release --config source.crates-io.replace-with=\"mirror\" \
    --config source.mirror.registry=\"sparse+https://mirrors.ustc.edu.cn/crates.io-index/\" -vv \
    --target=$(arch)-unknown-linux-musl \
    && mkdir out \
    && mv target/$(arch)-unknown-linux-musl/release/fontview ./out/fontview


FROM scratch
#FROM alpine
COPY --from=m /app/out/ /app/