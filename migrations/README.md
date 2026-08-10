# データベースマイグレーション

## 概要

このディレクトリには、データベーススキーマのマイグレーションファイルが含まれています。

## テーブル構造

### workflow_definitions

ワークフロー定義を保存するテーブルです。

- `id`: UUID（主キー）
- `application_type`: 申請タイプ（出張申請、経費申請など）
- `created_at`, `updated_at`: タイムスタンプ

### workflow_steps

ワークフローの各ステップを保存するテーブル（正規化済み）です。

- `id`: UUID（主キー）
- `workflow_definition_id`: ワークフロー定義ID（外部キー）
- `step_number`: ステップ番号（1から開始）
- `approver_role`: 承認者のロール（例: "上長", "開発本部長", "経理"）
- `is_required`: 必須ステップかどうか
- `created_at`, `updated_at`: タイムスタンプ

### 正規化のメリット

- データの整合性が保たれる
- クエリが簡単になる（JOINで取得）
- インデックスが効率的に使える
- 個別のステップを更新・削除しやすい
- ステップごとの追加情報を拡張しやすい

## マイグレーションの実行

```bash
make migrate
```

## 新しいマイグレーションの作成

```bash
make migrate-create
```

