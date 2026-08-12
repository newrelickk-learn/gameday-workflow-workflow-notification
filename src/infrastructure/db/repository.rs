use crate::domain::workflow::{WorkflowInstance, WorkflowStatus, WorkflowStep};
use anyhow::Result;
use rust_tracing_otel::{KeyValue, TelemetryManager};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, PaginatorTrait, QueryOrder};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use super::notification;
use super::workflow_definition;
use super::workflow_instance;
use super::workflow_step;

pub struct WorkflowRepository {
    db: DatabaseConnection,
    telemetry: Arc<TelemetryManager>,
}

impl WorkflowRepository {
    pub fn new(db: DatabaseConnection, telemetry: Arc<TelemetryManager>) -> Self {
        Self { db, telemetry }
    }

    /// db.client.operation.duration (Histogram) / db.client.response.returned_rows (Histogram)を記録する。
    /// New RelicのDatabasesページはこのメトリクス（apm.service.datastore.operation.duration）から
    /// 駆動されるため、スパンだけでは表示されない。
    fn record_db_metrics(&self, operation: &str, table: &str, duration: std::time::Duration, rows: Option<u64>) {
        let attributes = vec![
            KeyValue::new("db.system", "postgresql"),
            KeyValue::new("db.operation", operation.to_string()),
            KeyValue::new("db.sql.table", table.to_string()),
        ];
        self.telemetry
            .record_db_operation_duration(duration.as_secs_f64(), attributes.clone());
        if let Some(rows) = rows {
            self.telemetry.record_db_response_returned_rows(rows, attributes);
        }
    }

    #[tracing::instrument(skip(self), fields(
        otel.name = "SELECT workflow_instances",
        otel.kind = "client",
        db.system = "postgresql",
        db.operation = "SELECT",
        db.sql.table = "workflow_instances",
    ))]
    pub async fn get_workflow_instance_by_application_id(
        &self,
        application_id: &str,
    ) -> Result<Option<WorkflowInstance>> {
        let start = Instant::now();
        let instance = workflow_instance::Entity::find()
            .filter(workflow_instance::Column::ApplicationId.eq(application_id))
            .one(&self.db)
            .await?;
        self.record_db_metrics(
            "SELECT",
            "workflow_instances",
            start.elapsed(),
            Some(if instance.is_some() { 1 } else { 0 }),
        );

        Ok(instance.map(|m| m.into()))
    }

    #[tracing::instrument(skip(self), fields(
        otel.name = "INSERT workflow_instances",
        otel.kind = "client",
        db.system = "postgresql",
        db.operation = "INSERT",
        db.sql.table = "workflow_instances",
    ))]
    pub async fn create_workflow_instance(
        &self,
        application_id: &str,
        workflow_definition_id: Uuid,
        current_step: i32,
    ) -> Result<WorkflowInstance> {
        // 既存のインスタンスがある場合は削除（テスト用）
        if let Ok(Some(_)) = self.get_workflow_instance_by_application_id(application_id).await {
            let delete_start = Instant::now();
            workflow_instance::Entity::delete_many()
                .filter(workflow_instance::Column::ApplicationId.eq(application_id))
                .exec(&self.db)
                .await?;
            self.record_db_metrics("DELETE", "workflow_instances", delete_start.elapsed(), None);
        }

        let now = chrono::Utc::now();

        let active_model = workflow_instance::ActiveModel {
            id: Set(Uuid::new_v4()),
            application_id: Set(application_id.to_string()),
            workflow_definition_id: Set(workflow_definition_id),
            current_step: Set(current_step),
            status: Set("pending".to_string()),
            created_at: Set(sea_orm::prelude::DateTimeWithTimeZone::from(now)),
            updated_at: Set(sea_orm::prelude::DateTimeWithTimeZone::from(now)),
        };

        let start = Instant::now();
        let instance = active_model.insert(&self.db).await?;
        self.record_db_metrics("INSERT", "workflow_instances", start.elapsed(), Some(1));

        Ok(instance.into())
    }

    #[tracing::instrument(skip(self), fields(
        otel.name = "UPDATE workflow_instances",
        otel.kind = "client",
        db.system = "postgresql",
        db.operation = "UPDATE",
        db.sql.table = "workflow_instances",
    ))]
    pub async fn update_workflow_step(
        &self,
        application_id: &str,
        step: i32,
        status: &str,
    ) -> Result<()> {
        let instance = workflow_instance::Entity::find()
            .filter(workflow_instance::Column::ApplicationId.eq(application_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Workflow instance not found"))?;

        let mut active_model: workflow_instance::ActiveModel = instance.into();
        active_model.current_step = Set(step);
        active_model.status = Set(status.to_string());
        active_model.updated_at = Set(sea_orm::prelude::DateTimeWithTimeZone::from(chrono::Utc::now()));

        let start = Instant::now();
        active_model.update(&self.db).await?;
        self.record_db_metrics("UPDATE", "workflow_instances", start.elapsed(), Some(1));

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(
        otel.name = "SELECT workflow_steps",
        otel.kind = "client",
        db.system = "postgresql",
        db.operation = "SELECT",
        db.sql.table = "workflow_steps",
    ))]
    pub async fn get_total_steps(
        &self,
        workflow_definition_id: Uuid,
    ) -> Result<i32> {
        let start = Instant::now();
        let count = workflow_step::Entity::find()
            .filter(workflow_step::Column::WorkflowDefinitionId.eq(workflow_definition_id))
            .count(&self.db)
            .await?;
        self.record_db_metrics("SELECT", "workflow_steps", start.elapsed(), Some(count));

        Ok(count as i32)
    }

    #[tracing::instrument(skip(self), fields(
        otel.name = "SELECT workflow_steps",
        otel.kind = "client",
        db.system = "postgresql",
        db.operation = "SELECT",
        db.sql.table = "workflow_steps",
    ))]
    pub async fn get_workflow_steps(
        &self,
        workflow_definition_id: Uuid,
    ) -> Result<Vec<WorkflowStep>> {
        let start = Instant::now();
        let steps = workflow_step::Entity::find()
            .filter(workflow_step::Column::WorkflowDefinitionId.eq(workflow_definition_id))
            .order_by_asc(workflow_step::Column::StepNumber)
            .all(&self.db)
            .await?;
        self.record_db_metrics("SELECT", "workflow_steps", start.elapsed(), Some(steps.len() as u64));

        Ok(steps.into_iter().map(|s| s.into()).collect())
    }

    pub async fn get_workflow_definition_by_application_type(
        &self,
        application_type: &str,
    ) -> Result<Option<Uuid>> {
        // 後方互換性のため、company_id=1をデフォルトとして使用
        // （内部で呼ぶget_workflow_definition_by_application_type_and_company_idが計装されるため、
        // ここに独自のスパンは不要）
        self.get_workflow_definition_by_application_type_and_company_id(application_type, 1).await
    }

    #[tracing::instrument(skip(self), fields(
        otel.name = "SELECT workflow_definitions",
        otel.kind = "client",
        db.system = "postgresql",
        db.operation = "SELECT",
        db.sql.table = "workflow_definitions",
    ))]
    pub async fn get_workflow_definition_by_application_type_and_company_id(
        &self,
        application_type: &str,
        company_id: i32,
    ) -> Result<Option<Uuid>> {
        let start = Instant::now();
        let definition = workflow_definition::Entity::find()
            .filter(workflow_definition::Column::ApplicationType.eq(application_type))
            .filter(workflow_definition::Column::CompanyId.eq(company_id))
            .one(&self.db)
            .await?;
        self.record_db_metrics(
            "SELECT",
            "workflow_definitions",
            start.elapsed(),
            Some(if definition.is_some() { 1 } else { 0 }),
        );

        Ok(definition.map(|d| d.id))
    }

    #[tracing::instrument(skip(self, body), fields(
        otel.name = "INSERT notifications",
        otel.kind = "client",
        db.system = "postgresql",
        db.operation = "INSERT",
        db.sql.table = "notifications",
    ))]
    pub async fn create_notification(
        &self,
        notification_type: &str,
        channel: &str,
        recipient_id: &str,
        recipient_email: Option<&str>,
        subject: &str,
        body: &str,
    ) -> Result<Uuid> {
        let now = chrono::Utc::now();

        let active_model = notification::ActiveModel {
            id: Set(Uuid::new_v4()),
            notification_type: Set(notification_type.to_string()),
            channel: Set(channel.to_string()),
            recipient_id: Set(recipient_id.to_string()),
            recipient_email: Set(recipient_email.map(|s| s.to_string())),
            subject: Set(subject.to_string()),
            body: Set(body.to_string()),
            sent_at: Set(Some(sea_orm::prelude::DateTimeWithTimeZone::from(now))),
            created_at: Set(sea_orm::prelude::DateTimeWithTimeZone::from(now)),
        };

        let start = Instant::now();
        let notification = active_model.insert(&self.db).await?;
        self.record_db_metrics("INSERT", "notifications", start.elapsed(), Some(1));

        Ok(notification.id)
    }

    #[tracing::instrument(skip(self), fields(
        otel.name = "SELECT notifications",
        otel.kind = "client",
        db.system = "postgresql",
        db.operation = "SELECT",
        db.sql.table = "notifications",
    ))]
    pub async fn get_notifications_by_recipient_id(
        &self,
        recipient_id: &str,
    ) -> Result<Vec<crate::domain::notification::Notification>> {
        let start = Instant::now();
        let notifications = notification::Entity::find()
            .filter(notification::Column::RecipientId.eq(recipient_id))
            .order_by_desc(notification::Column::CreatedAt)
            .all(&self.db)
            .await?;
        self.record_db_metrics(
            "SELECT",
            "notifications",
            start.elapsed(),
            Some(notifications.len() as u64),
        );

        Ok(notifications.into_iter().map(|n| n.into()).collect())
    }
}

impl From<workflow_instance::Model> for WorkflowInstance {
    fn from(model: workflow_instance::Model) -> Self {
        let status = match model.status.as_str() {
            "pending" => WorkflowStatus::Pending,
            "in_progress" => WorkflowStatus::InProgress,
            "completed" => WorkflowStatus::Completed,
            "rejected" => WorkflowStatus::Rejected,
            "error" => WorkflowStatus::Error,
            _ => WorkflowStatus::Pending,
        };

        WorkflowInstance {
            id: model.id,
            application_id: model.application_id,
            workflow_definition_id: model.workflow_definition_id,
            current_step: model.current_step,
            status,
            created_at: chrono::DateTime::<chrono::Utc>::from(model.created_at),
            updated_at: chrono::DateTime::<chrono::Utc>::from(model.updated_at),
        }
    }
}

impl From<workflow_step::Model> for WorkflowStep {
    fn from(model: workflow_step::Model) -> Self {
        WorkflowStep {
            step_number: model.step_number,
            approver_role: model.approver_role,
            is_required: model.is_required,
        }
    }
}

impl From<notification::Model> for crate::domain::notification::Notification {
    fn from(model: notification::Model) -> Self {
        use crate::domain::notification::{NotificationChannel, NotificationType};

        let notification_type = match model.notification_type.as_str() {
            "ApprovalRequest" => NotificationType::ApprovalRequest,
            "ApprovalCompleted" => NotificationType::ApprovalCompleted,
            "ApprovalRejected" => NotificationType::ApprovalRejected,
            "WorkflowCompleted" => NotificationType::WorkflowCompleted,
            _ => NotificationType::ApprovalRequest,
        };

        let channel = match model.channel.as_str() {
            "Email" => NotificationChannel::Email,
            "Slack" => NotificationChannel::Slack,
            _ => NotificationChannel::Email,
        };

        crate::domain::notification::Notification {
            id: model.id,
            notification_type,
            channel,
            recipient_id: model.recipient_id,
            recipient_email: model.recipient_email,
            subject: model.subject,
            body: model.body,
            sent_at: model.sent_at.map(|dt| chrono::DateTime::<chrono::Utc>::from(dt)),
            created_at: chrono::DateTime::<chrono::Utc>::from(model.created_at),
        }
    }
}
