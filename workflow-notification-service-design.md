# Workflow & Notification Service (Rust) プロジェクト設計

## リポジトリ名
`gameday-workflow-workflow-notification`

## サービスの責務

このサービスは、申請承認ワークフローにおける**ワークフロー実行エンジン**と**通知機能**を担当します。

### ワークフロー機能

1. **ワークフロー定義の管理**
   - 申請タイプ（出張申請、経費申請、プロモーション申請）ごとの承認フロー定義を管理
   - ワークフロー定義の作成・更新・削除
   - 承認ステップの定義（上長承認 → 開発本部長承認 → 経理承認など）

2. **ワークフロー実行エンジン**
   - 申請提出時にワークフローインスタンスを作成・開始
   - ワークフロー実行状態の管理（進行中、完了、エラーなど）
   - 現在の承認ステップの追跡

3. **状態遷移管理**
   - ワークフローの各ステップ間の状態遷移を管理
   - 承認完了時の次のステップへの自動遷移
   - ワークフロー完了時の最終状態への遷移

4. **承認バリデーション（同期処理）**
   - Application & Approval Serviceから承認前に呼び出される
   - 承認可能かどうかの検証（現在のステップ、承認者の権限など）
   - 次のステップの判定（最終承認か、次の承認者へか）
   - ワークフロー状態の確認と更新準備
   - 同期APIエンドポイント: `POST /workflows/validate-approval`

5. **承認ルーティング**
   - 申請タイプに基づいて、誰が承認すべきかを判定
   - ユーザーのロール（上長、開発本部長、経理担当）に基づく承認者の特定
   - 承認依頼の作成とApplication & Approval Serviceへの通知

### 通知機能

1. **通知送信**
   - 承認依頼時の承認者への通知（メール、Slack等）
   - 承認完了時の申請者への通知
   - ワークフロー完了時の関係者への通知
   - 承認却下時の申請者への通知

2. **通知履歴管理**
   - 送信した通知の履歴を記録
   - 通知送信の成功・失敗の追跡
   - 通知設定（通知先、通知方法）の管理

### 処理方式

#### 同期処理（REST/gRPC API）

- **承認バリデーション**
  - Application & Approval Serviceから承認前に呼び出される
  - 承認可能かどうかの検証と次のステップの判定を即座に返却
  - APIエンドポイント: `POST /workflows/validate-approval`

#### 非同期処理（Kafkaイベント）

- **Kafkaイベントの購読**
  - `application.submitted`: 申請提出時にワークフローを開始
  - `approval.approved`: 承認完了時に通知送信（バリデーションは同期で完了済み）
  - `approval.rejected`: 承認却下時にワークフローを終了し通知送信
  - `workflow.completed`: ワークフロー完了時に通知送信

- **Kafkaイベントの発行**
  - `workflow.started`: ワークフロー開始を通知
  - `workflow.completed`: ワークフロー完了を通知
  - `workflow.step.completed`: 各ステップ完了を通知

### データ管理

- **ワークフロー定義**: 申請タイプごとの承認フロー定義
- **ワークフローインスタンス**: 実行中のワークフローの状態
- **通知履歴**: 送信した通知の記録
- **通知設定**: 通知先や通知方法の設定

## 技術スタック
- Rust 1.70+
- Actix Web
- SQLx
- PostgreSQL
- rdkafka (Kafka client)

## プロジェクト構成

```
gameday-workflow-workflow-notification-service/
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── deploy.yml
├── src/
│   ├── api/
│   │   ├── handlers/
│   │   │   ├── workflow.rs
│   │   │   └── notification.rs
│   │   └── routes.rs
│   ├── domain/
│   │   ├── workflow.rs
│   │   └── notification.rs
│   ├── infrastructure/
│   │   ├── db/
│   │   │   └── repository.rs
│   │   ├── kafka/
│   │   │   ├── consumer.rs
│   │   │   └── producer.rs
│   │   └── email/
│   │       └── sender.rs
│   ├── services/
│   │   ├── workflow_service.rs
│   │   └── notification_service.rs
│   └── main.rs
├── tests/
│   ├── integration/
│   └── unit/
├── migrations/
├── Dockerfile
├── .dockerignore
├── .gitignore
├── Cargo.toml
└── README.md
```

## スタブ実装の設計

### テスト用のモック実装

```rust
// tests/mocks/mod.rs
use crate::domain::workflow::WorkflowDefinition;
use crate::services::workflow_service::WorkflowService;

pub struct MockWorkflowService;

impl WorkflowService for MockWorkflowService {
    fn start_workflow(&self, application_id: &str) -> Result<String, String> {
        Ok(format!("workflow-{}", application_id))
    }
}
```

## 単体テスト構成

```rust
// tests/unit/workflow_service_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_start_workflow() {
        let service = MockWorkflowService;
        let result = service.start_workflow("app-1");
        assert!(result.is_ok());
    }
}
```

## GitHub Actions設定

### CI/CDパイプライン (.github/workflows/ci.yml)

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Run tests
        run: cargo test --verbose
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
      
      - name: Build
        run: cargo build --release
```

### デプロイワークフロー (.github/workflows/deploy.yml)

```yaml
name: Deploy to EKS

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ap-northeast-1
      
      - name: Login to Amazon ECR
        uses: aws-actions/amazon-ecr-login@v2
      
      - name: Build, tag, and push image
        env:
          ECR_REGISTRY: ${{ steps.login-ecr.outputs.registry }}
          ECR_REPOSITORY: gameday-workflow-workflow-notification-service
          IMAGE_TAG: ${{ github.sha }}
        run: |
          docker build -t $ECR_REGISTRY/$ECR_REPOSITORY:$IMAGE_TAG .
          docker push $ECR_REGISTRY/$ECR_REPOSITORY:$IMAGE_TAG
      
      - name: Update kubeconfig
        run: |
          aws eks update-kubeconfig --name gameday-workflow-cluster --region ap-northeast-1
      
      - name: Deploy to EKS
        run: |
          kubectl set image deployment/workflow-notification-service workflow-notification-service=$ECR_REGISTRY/$ECR_REPOSITORY:$IMAGE_TAG -n gameday-workflow
          kubectl rollout status deployment/workflow-notification-service -n gameday-workflow
```

## Dockerfile

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/workflow-notification-service .
CMD ["./workflow-notification-service"]
```

