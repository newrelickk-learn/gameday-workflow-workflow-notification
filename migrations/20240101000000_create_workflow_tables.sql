CREATE TABLE IF NOT EXISTS workflow_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    application_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(application_type)
);

CREATE TABLE IF NOT EXISTS workflow_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_definition_id UUID NOT NULL REFERENCES workflow_definitions(id) ON DELETE CASCADE,
    step_number INTEGER NOT NULL,
    approver_role VARCHAR(255) NOT NULL,
    is_required BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(workflow_definition_id, step_number)
);

CREATE TABLE IF NOT EXISTS workflow_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id VARCHAR(255) NOT NULL,
    workflow_definition_id UUID NOT NULL REFERENCES workflow_definitions(id),
    current_step INTEGER NOT NULL DEFAULT 1,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(application_id)
);

CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_type VARCHAR(50) NOT NULL,
    channel VARCHAR(50) NOT NULL,
    recipient_id VARCHAR(255) NOT NULL,
    recipient_email VARCHAR(255),
    subject VARCHAR(500) NOT NULL,
    body TEXT NOT NULL,
    sent_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS notification_settings (
    user_id VARCHAR(255) PRIMARY KEY,
    email_enabled BOOLEAN NOT NULL DEFAULT true,
    slack_enabled BOOLEAN NOT NULL DEFAULT false,
    slack_webhook_url VARCHAR(500),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflow_instances_application_id ON workflow_instances(application_id);
CREATE INDEX IF NOT EXISTS idx_workflow_instances_status ON workflow_instances(status);
CREATE INDEX IF NOT EXISTS idx_workflow_steps_definition_id ON workflow_steps(workflow_definition_id);
CREATE INDEX IF NOT EXISTS idx_workflow_steps_step_number ON workflow_steps(workflow_definition_id, step_number);
CREATE INDEX IF NOT EXISTS idx_notifications_recipient_id ON notifications(recipient_id);
CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at);

INSERT INTO workflow_definitions (application_type) VALUES
('BusinessTrip')
ON CONFLICT (application_type) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 1, 'エンジニア', true FROM workflow_definitions WHERE application_type = 'BusinessTrip'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 2, '上長', true FROM workflow_definitions WHERE application_type = 'BusinessTrip'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 3, '本部長', true FROM workflow_definitions WHERE application_type = 'BusinessTrip'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_definitions (application_type) VALUES
('Expense')
ON CONFLICT (application_type) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 1, 'エンジニア', true FROM workflow_definitions WHERE application_type = 'Expense'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 2, '上長', true FROM workflow_definitions WHERE application_type = 'Expense'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 3, '経理', true FROM workflow_definitions WHERE application_type = 'Expense'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_definitions (application_type) VALUES
('Vacation')
ON CONFLICT (application_type) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 1, 'エンジニア', true FROM workflow_definitions WHERE application_type = 'Vacation'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 2, '上長', true FROM workflow_definitions WHERE application_type = 'Vacation'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_definitions (application_type) VALUES
('Promotion')
ON CONFLICT (application_type) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 1, '上長', true FROM workflow_definitions WHERE application_type = 'Promotion'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;

INSERT INTO workflow_steps (workflow_definition_id, step_number, approver_role, is_required)
SELECT id, 2, '本部長', true FROM workflow_definitions WHERE application_type = 'Promotion'
ON CONFLICT (workflow_definition_id, step_number) DO NOTHING;


