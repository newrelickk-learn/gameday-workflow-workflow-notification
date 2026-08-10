# マルチステージビルド
FROM rust:latest as builder

# ビルド依存関係をインストール
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 依存関係を先にコピーしてキャッシュを活用
COPY Cargo.toml ./
# Cargo.lockが存在する場合のみコピー
COPY Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# ソースコードをコピーしてビルド
COPY . .
RUN touch src/main.rs
RUN cargo build --release

# ランタイムイメージ
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ビルド済みバイナリをコピー
COPY --from=builder /app/target/release/workflow-notification-service .

# ポートを公開
EXPOSE 8003

CMD ["./workflow-notification-service"]
