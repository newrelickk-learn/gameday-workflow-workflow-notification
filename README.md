# Workflow & Notification Service

GameDay Workflow システムのワークフロー・通知管理サービス

## 概要

このサービスは、申請承認ワークフローにおける**ワークフロー実行エンジン**と**通知機能**を担当します。

### 主な機能

- ワークフロー定義の管理
- ワークフロー実行エンジン
- 状態遷移管理
- 承認バリデーション（同期処理）
- 承認ルーティング
- 通知送信（メール、Slack等）
- 通知履歴管理

## 技術スタック

- Rust 1.70+
- Actix Web
- SeaORM
- PostgreSQL
- rdkafka (Kafka client)

## セットアップ

### 必要な環境

- Rust 1.70以上
- Make
- PostgreSQL（他のプロジェクトのdocker-composeで起動）
- Docker（オプション）

### 環境変数

`.env`ファイルを作成し、以下の環境変数を設定してください：

```bash
cp .env.example .env
```

`.env`ファイルを編集して、データベース接続情報を設定してください：

```env
PORT=8003
DATABASE_URL=postgresql://gameday_user:gameday_password@postgres:5432/workflow_notification
RUST_LOG=info
```

### ツールのインストール

```bash
make install-tools
```

### データベースマイグレーション

```bash
make migrate
```

### ビルド

```bash
make build
```

### 実行

```bash
make run
```

### APIテスト

```bash
make test-api
```

## API エンドポイント

### 承認バリデーション

```
POST /api/v1/workflows/validate-approval
```

申請承認サービスから承認前に呼び出されるエンドポイント。承認可能かどうかの検証と次のステップの判定を返却します。

詳細は `specs/workflow-service-openapi.yaml` を参照してください。

## 開発

### テスト実行

```bash
make test
```

### コードチェック

```bash
make check
```

### コードフォーマット

```bash
make fmt
```

### マイグレーション作成

```bash
make migrate-create
```

## Docker

すべてのDockerビルドは`docker-compose`を使用します。

### ローカル用（ARM64）ビルド

```bash
make docker-build-local
```

または直接：

```bash
BUILD_PLATFORM=linux/arm64 IMAGE_TAG=local docker-compose build
```

### 本番用（AMD64）ビルド

```bash
make docker-build-prod
```

または直接：

```bash
IMAGE_TAG=prod docker-compose -f docker-compose.prod.yml build
```

### マルチアーキテクチャビルド

```bash
make docker-build
```

### docker-composeでの実行

```bash
# ビルドと起動
make docker-build-local
make docker-up

# ログ確認
make docker-logs

# 停止
make docker-down

# コンテナ状態確認
make docker-ps
```

注意: PostgreSQLは他のプロジェクトのdocker-composeで起動していることを前提としています。

