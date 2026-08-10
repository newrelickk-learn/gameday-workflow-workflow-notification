use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Pending,
    InProgress,
    Completed,
    Rejected,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApplicationType {
    BusinessTrip,
    Expense,
    // 経費精算（海外出張後の高額な経費など）。Expenseとは別のワークフロー定義
    // （直属マネージャー承認 → 経理部承認の2段階承認）を持つ。
    ExpenseSettlement,
    Promotion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: Uuid,
    pub application_type: ApplicationType,
    pub steps: Vec<WorkflowStep>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub step_number: i32,
    pub approver_role: String,
    pub is_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: Uuid,
    pub application_id: String,
    pub workflow_definition_id: Uuid,
    pub current_step: i32,
    pub status: WorkflowStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalValidation {
    pub approval_id: String,
    pub application_id: String,
    pub approver_id: String,
    pub current_step: i32,
    pub is_final_step: bool,
    pub next_step: Option<i32>,
}

