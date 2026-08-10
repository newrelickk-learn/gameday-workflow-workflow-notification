-- 経費精算（2段階承認）ワークフロー定義の追加
--
-- 背景:
-- 通常の経費申請（Expense）は1段階承認（直属マネージャーのみ）のままにする。
-- 一方、海外出張後の経費精算のような高額な経費（航空券・宿泊費など）は、
-- 社内規定により「直属マネージャー承認 → 経理部承認」の2段階承認が必要になる。
-- この2段階承認は、既存の Expense とは別の application_type
-- （ExpenseSettlement）を持つワークフロー定義として表現する
-- （どちらの application_type を使うかは、金額基準でアプリケーション側が判定する）。

-- 1. 経費申請（Expense）を1段階承認（エンジニア申請 → 上長承認のみ）に変更
--    既存データでは経理承認（step_number=3）が含まれていたため、これを削除する。
DELETE FROM workflow_steps
WHERE step_number = 3
  AND workflow_definition_id IN (
      SELECT id FROM workflow_definitions WHERE application_type = 'Expense'
  );

UPDATE workflow_definitions
SET updated_at = NOW()
WHERE application_type = 'Expense';

-- 2. 経費精算（ExpenseSettlement）のワークフロー定義を追加（各会社ごと）
--    エンジニア申請 → 上長承認 → 経理承認（3ステップ、承認は上長・経理の2段階）
INSERT INTO workflow_definitions (application_type, company_id)
SELECT 'ExpenseSettlement', generate_series(1, 50)
ON CONFLICT (application_type, company_id) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT wd.id, 1, 'エンジニア', true
FROM workflow_definitions wd
WHERE wd.application_type = 'ExpenseSettlement'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT wd.id, 2, '上長', true
FROM workflow_definitions wd
WHERE wd.application_type = 'ExpenseSettlement'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT wd.id, 3, '経理', true
FROM workflow_definitions wd
WHERE wd.application_type = 'ExpenseSettlement'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;
