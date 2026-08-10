.PHONY: help go-build run test test-api clean build docker-build-local docker-build-prod up docker-down docker-logs docker-ps migrate migrate-create migrate-revert check fmt install-tools

# デフォルトターゲット
.DEFAULT_GOAL := help

# 環境変数
PORT ?= 8003
DATABASE_URL ?= postgresql://gameday_user:gameday_password@postgres:5432/gameday_workflow_notification
RUST_LOG ?= info

help: ## このヘルプメッセージを表示
	@echo "利用可能なコマンド:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

go-build: ## プロジェクトをビルド
	cargo build --release

run: ## アプリケーションを実行
	RUST_LOG=$(RUST_LOG) PORT=$(PORT) DATABASE_URL=$(DATABASE_URL) cargo run

test: ## ユニットテストを実行
	cargo test

test-api: ## curlを使用してAPIテストを実行
	@echo "=== APIテスト開始 ==="
	@./scripts/test-api.sh

clean: ## ビルド成果物をクリーンアップ
	cargo clean

build: ## Dockerイメージをビルド（マルチアーキテクチャ）
	BUILD_PLATFORM=linux/arm64 IMAGE_TAG=local docker-compose build
	
docker-build-local: ## Dockerイメージをビルド（ローカル用 ARM64）
	BUILD_PLATFORM=linux/arm64 IMAGE_TAG=local docker-compose build

docker-build-prod: ## Dockerイメージをビルド（本番用 AMD64）
	IMAGE_TAG=prod docker-compose -f docker-compose.prod.yml build

up: ## docker-composeでサービスを起動
	docker-compose up

docker-down: ## docker-composeでサービスを停止
	docker-compose down

docker-logs: ## docker-composeのログを表示
	docker-compose logs -f workflow-notification-service

docker-ps: ## docker-composeのコンテナ状態を表示
	docker-compose ps

migrate: ## データベースマイグレーションを実行
	sqlx migrate run

migrate-create: ## 新しいマイグレーションファイルを作成
	@read -p "マイグレーション名を入力してください: " name; \
	sqlx migrate add $$name

migrate-revert: ## 最後のマイグレーションをロールバック
	sqlx migrate revert

check: ## コードチェック（clippy + fmt）
	cargo clippy -- -D warnings
	cargo fmt --check

fmt: ## コードフォーマット
	cargo fmt

install-tools: ## 必要なツールをインストール
	cargo install sqlx-cli --features postgres

