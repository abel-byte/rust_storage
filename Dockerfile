####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN rustup component add rustfmt
RUN apt update && apt install -y musl musl-tools musl-dev protobuf-compiler cmake pkg-config openssl libssl-dev git clang llvm
RUN update-ca-certificates

# Static linking for C++ code
RUN ln -s "/usr/bin/g++" "/usr/bin/musl-g++"

# Create appuser
ENV USER=app
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /app

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
FROM alpine

# 增加镜像
RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.ustc.edu.cn/g' /etc/apk/repositories

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

# Copy our build
COPY --from=builder /app/config/ ./
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rust_storage ./

# Use an unprivileged user.
USER app:app

CMD ["/app/rust_storage"]
