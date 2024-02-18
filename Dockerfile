FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt -y update
RUN apt install -y musl-tools musl-dev
RUN apt-get install -y build-essential
RUN apt install -y gcc-x86-64-linux-gnu
WORKDIR /app
COPY . ./

RUN cargo build --target x86_64-unknown-linux-musl --release

# FROM ekidd/rust-musl-builder AS builder
#
# COPY --chown=rust:rust . ./
# RUN cargo build --release

FROM scratch
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/qna ./
COPY --from=builder /app/.env ./
# Executing the binary
ENTRYPOINT ["/app/qna"]
