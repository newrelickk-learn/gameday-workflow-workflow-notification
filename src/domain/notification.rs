use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    ApprovalRequest,
    ApprovalCompleted,
    ApprovalRejected,
    WorkflowCompleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Slack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub notification_type: NotificationType,
    pub channel: NotificationChannel,
    pub recipient_id: String,
    pub recipient_email: Option<String>,
    pub subject: String,
    pub body: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub user_id: String,
    pub email_enabled: bool,
    pub slack_enabled: bool,
    pub slack_webhook_url: Option<String>,
}

