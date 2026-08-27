
DELETE FROM workflow_steps
WHERE step_number = 3
  AND workflow_definition_id IN (
      SELECT id FROM workflow_definitions WHERE application_type = 'Expense'
  );

UPDATE workflow_definitions
SET updated_at = NOW()
WHERE application_type = 'Expense';

INSERT INTO workflow_definitions (application_type, company_id)
SELECT 'ExpenseSettlement', generate_series(1::integer, 50::integer)
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
