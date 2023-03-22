ARG BASE_IMAGE=rust:latest

FROM $BASE_IMAGE as planner
WORKDIR /app
RUN echo 'deb http://ftp.cn.debian.org/debian/ stable main' > /etc/apt/sources.list
RUN apt update && apt install -y protobuf-compiler clang pkg-config libssl-dev
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM $BASE_IMAGE as cacher
WORKDIR /app
RUN echo 'deb http://ftp.cn.debian.org/debian/ stable main' > /etc/apt/sources.list
RUN apt update && apt install -y protobuf-compiler clang pkg-config libssl-dev
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM $BASE_IMAGE as builder
WORKDIR /app
RUN echo 'deb http://ftp.cn.debian.org/debian/ stable main' > /etc/apt/sources.list
RUN apt update && apt install -y protobuf-compiler clang pkg-config libssl-dev
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
# `cargo build` doesn't work in static linking, need `cargo install`
RUN cargo install --path .

FROM $BASE_IMAGE
COPY --from=builder /app/config/ ./config
COPY --from=builder /usr/local/cargo/bin/rust_storage .
CMD ["./rust_storage"]
