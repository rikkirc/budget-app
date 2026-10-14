# Build from the repository root (the server needs ../vault-core):  docker build -f server/Dockerfile -t vault-server .
FROM rust:1-bookworm AS build
WORKDIR /src
COPY vault-core vault-core
COPY server server
RUN cargo build --release --locked --manifest-path server/Cargo.toml

FROM debian:bookworm-slim
COPY --from=build /src/server/target/release/vault-server /usr/local/bin/vault-server
ENV VAULT_DATA_DIR=/data PORT=8080
VOLUME /data
EXPOSE 8080
CMD ["vault-server"]
