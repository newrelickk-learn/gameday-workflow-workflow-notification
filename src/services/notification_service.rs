use crate::domain::notification::{Notification, NotificationType};
use crate::infrastructure::db::WorkflowRepository;
use anyhow::Result;
use tracing::Instrument;

#[async_trait::async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_notification(
        &self,
        notification_type: NotificationType,
        recipient_id: &str,
        subject: &str,
        body: &str,
    ) -> Result<()>;

    async fn get_notification_history(
        &self,
        recipient_id: &str,
    ) -> Result<Vec<Notification>>;
}

pub struct NotificationServiceImpl {
    repository: WorkflowRepository,
}

impl NotificationServiceImpl {
    pub fn new(repository: WorkflowRepository) -> Self {
        Self { repository }
    }
}

#[async_trait::async_trait]
impl NotificationService for NotificationServiceImpl {
    async fn send_notification(
        &self,
        notification_type: NotificationType,
        recipient_id: &str,
        subject: &str,
        body: &str,
    ) -> Result<()> {
        // デモ用: 通知履歴に記録するのみ（実際のメール/Slack送信は行わない）
        let notification_type_str = match notification_type {
            NotificationType::ApprovalRequest => "ApprovalRequest",
            NotificationType::ApprovalCompleted => "ApprovalCompleted",
            NotificationType::ApprovalRejected => "ApprovalRejected",
            NotificationType::WorkflowCompleted => "WorkflowCompleted",
        };

        let channel = "Email"; // デモ用: Emailとして記録

        // NotificationType/NotificationChannelはDisplayを実装していないため`?`(Debug)で記録する。
        // recipient_idは&strでDisplayが使えるため`%`で記録する。
        let span = tracing::info_span!(
            "send_notification",
            notification.r#type = ?notification_type,
            notification.recipientId = %recipient_id,
            notification.channel = %channel,
        );

        async move {
            self.repository
                .create_notification(
                    notification_type_str,
                    channel,
                    recipient_id,
                    None, // recipient_emailは省略
                    subject,
                    body,
                )
                .await?;

            tracing::info!(
                "Notification saved: type={:?}, recipient={}, subject={}",
                notification_type,
                recipient_id,
                subject
            );

            Ok(())
        }
        .instrument(span)
        .await
    }

    async fn get_notification_history(
        &self,
        recipient_id: &str,
    ) -> Result<Vec<Notification>> {
        self.repository
            .get_notifications_by_recipient_id(recipient_id)
            .await
    }
}

