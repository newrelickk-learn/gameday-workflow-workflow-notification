-- ワークフロー定義テーブルにcompany_idカラムを追加
-- 各会社ごとにワークフロー定義を持つことができるようにする

-- 1. workflow_definitionsテーブルにcompany_idカラムを追加
ALTER TABLE workflow_definitions 
ADD COLUMN IF NOT EXISTS company_id INTEGER;

-- 2. 既存のワークフロー定義にcompany_id=1を設定（デフォルト）
UPDATE workflow_definitions 
SET company_id = 1 
WHERE company_id IS NULL;

-- 3. company_idをNOT NULLに設定（既存データに1を設定した後）
ALTER TABLE workflow_definitions 
ALTER COLUMN company_id SET NOT NULL;

-- 4. 複合ユニーク制約を追加（application_typeとcompany_idの組み合わせで一意）
ALTER TABLE workflow_definitions 
DROP CONSTRAINT IF EXISTS workflow_definitions_application_type_key;

ALTER TABLE workflow_definitions 
ADD CONSTRAINT workflow_definitions_application_type_company_id_key 
UNIQUE (application_type, company_id);

-- 5. インデックスを追加（company_idでの検索を高速化）
CREATE INDEX IF NOT EXISTS idx_workflow_definitions_company_id 
ON workflow_definitions(company_id);

-- 6. 各会社（1-50）ごとにワークフロー定義を作成
-- 出張申請（BusinessTrip）
INSERT INTO workflow_definitions (application_type, company_id)
SELECT 'BusinessTrip', generate_series(1, 50)
ON CONFLICT (application_type, company_id) DO NOTHING;

-- 経費申請（Expense）
INSERT INTO workflow_definitions (application_type, company_id)
SELECT 'Expense', generate_series(1, 50)
ON CONFLICT (application_type, company_id) DO NOTHING;

-- 休暇申請（Vacation）
INSERT INTO workflow_definitions (application_type, company_id)
SELECT 'Vacation', generate_series(1, 50)
ON CONFLICT (application_type, company_id) DO NOTHING;

-- プロモーション申請（Promotion）
INSERT INTO workflow_definitions (application_type, company_id)
SELECT 'Promotion', generate_series(1, 50)
ON CONFLICT (application_type, company_id) DO NOTHING;

-- 7. 各会社ごとにワークフローステップを作成
-- 出張申請のワークフローステップ（各会社ごと）
INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT wd.id, 1, 'エンジニア', true
FROM workflow_definitions wd
WHERE wd.application_type = 'BusinessTrip'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT wd.id, 2, '上長', true
FROM workflow_definitions wd
WHERE wd.application_type = 'BusinessTrip'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT wd.id, 3, '本部長', true
FROM workflow_definitions wd
WHERE wd.application_type = 'BusinessTrip'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

-- 経費申請のワークフローステップ（各会社ごと）
INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT wd.id, 1, 'エンジニア', true
FROM workflow_definitions wd
WHERE wd.application_type = 'Expense'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT wd.id, 2, '上長', true
FROM workflow_definitions wd
WHERE wd.application_type = 'Expense'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT wd.id, 3, '経理', true
FROM workflow_definitions wd
WHERE wd.application_type = 'Expense'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

-- 休暇申請のワークフローステップ（各会社ごと）
INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT wd.id, 1, 'エンジニア', true
FROM workflow_definitions wd
WHERE wd.application_type = 'Vacation'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT wd.id, 2, '上長', true
FROM workflow_definitions wd
WHERE wd.application_type = 'Vacation'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

-- プロモーション申請のワークフローステップ（各会社ごと）
INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT wd.id, 1, '上長', true
FROM workflow_definitions wd
WHERE wd.application_type = 'Promotion'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT wd.id, 2, '本部長', true
FROM workflow_definitions wd
WHERE wd.application_type = 'Promotion'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

