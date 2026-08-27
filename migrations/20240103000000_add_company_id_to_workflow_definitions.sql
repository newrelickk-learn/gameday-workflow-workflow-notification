
ALTER TABLE workflow_definitions
ADD COLUMN IF NOT EXISTS company_id INTEGER;

UPDATE workflow_definitions
SET company_id = 1
WHERE company_id IS NULL;

ALTER TABLE workflow_definitions
ALTER COLUMN company_id SET NOT NULL;

ALTER TABLE workflow_definitions
DROP CONSTRAINT IF EXISTS workflow_definitions_application_type_key;

ALTER TABLE workflow_definitions
ADD CONSTRAINT workflow_definitions_application_type_company_id_key
UNIQUE (application_type, company_id);

CREATE INDEX IF NOT EXISTS idx_workflow_definitions_company_id
ON workflow_definitions(company_id);

INSERT INTO workflow_definitions (application_type, company_id)
SELECT 'BusinessTrip', generate_series(1::integer, 50::integer)
ON CONFLICT (application_type, company_id) DO NOTHING;

INSERT INTO workflow_definitions (application_type, company_id)
SELECT 'Expense', generate_series(1::integer, 50::integer)
ON CONFLICT (application_type, company_id) DO NOTHING;

INSERT INTO workflow_definitions (application_type, company_id)
SELECT 'Vacation', generate_series(1::integer, 50::integer)
ON CONFLICT (application_type, company_id) DO NOTHING;

INSERT INTO workflow_definitions (application_type, company_id)
SELECT 'Promotion', generate_series(1::integer, 50::integer)
ON CONFLICT (application_type, company_id) DO NOTHING;

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

