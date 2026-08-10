-- ワークフロー定義の更新と追加
-- 既存のワークフロー定義を更新し、新しいワークフロー定義を追加します

-- 1. 出張申請（BusinessTrip）の更新
-- エンジニア申請 → 上長承認 → 本部長最終承認（3ステップ）
DELETE FROM workflow_steps 
WHERE workflow_definition_id IN (
    SELECT id FROM workflow_definitions WHERE application_type = 'BusinessTrip'
);

UPDATE workflow_definitions 
SET updated_at = NOW()
WHERE application_type = 'BusinessTrip';

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 1, 'エンジニア', true FROM workflow_definitions WHERE application_type = 'BusinessTrip';

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 2, '上長', true FROM workflow_definitions WHERE application_type = 'BusinessTrip';

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 3, '本部長', true FROM workflow_definitions WHERE application_type = 'BusinessTrip';

-- 2. 経費申請（Expense）の更新
-- エンジニア申請 → 上長承認 → 経理承認（3ステップ）
DELETE FROM workflow_steps 
WHERE workflow_definition_id IN (
    SELECT id FROM workflow_definitions WHERE application_type = 'Expense'
);

UPDATE workflow_definitions 
SET updated_at = NOW()
WHERE application_type = 'Expense';

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 1, 'エンジニア', true FROM workflow_definitions WHERE application_type = 'Expense';

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 2, '上長', true FROM workflow_definitions WHERE application_type = 'Expense';

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 3, '経理', true FROM workflow_definitions WHERE application_type = 'Expense';

-- 3. 休暇申請（Vacation）の追加
-- エンジニア申請 → 上長承認（2ステップ）
INSERT INTO workflow_definitions (application_type) VALUES
('Vacation')
ON CONFLICT (application_type) DO UPDATE SET updated_at = NOW();

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 1, 'エンジニア', true FROM workflow_definitions WHERE application_type = 'Vacation'
ON CONFLICT (workflow_definition_id, step_number) DO UPDATE 
SET approver_role = EXCLUDED.approver_role, updated_at = NOW();

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 2, '上長', true FROM workflow_definitions WHERE application_type = 'Vacation'
ON CONFLICT (workflow_definition_id, step_number) DO UPDATE 
SET approver_role = EXCLUDED.approver_role, updated_at = NOW();

-- 4. プロモーション申請（Promotion）の追加
-- 上長申請 → 本部長承認（2ステップ）
INSERT INTO workflow_definitions (application_type) VALUES
('Promotion')
ON CONFLICT (application_type) DO UPDATE SET updated_at = NOW();

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 1, '上長', true FROM workflow_definitions WHERE application_type = 'Promotion'
ON CONFLICT (workflow_definition_id, step_number) DO UPDATE 
SET approver_role = EXCLUDED.approver_role, updated_at = NOW();

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 2, '本部長', true FROM workflow_definitions WHERE application_type = 'Promotion'
ON CONFLICT (workflow_definition_id, step_number) DO UPDATE 
SET approver_role = EXCLUDED.approver_role, updated_at = NOW();

