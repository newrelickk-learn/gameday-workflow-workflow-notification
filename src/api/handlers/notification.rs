use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tracing;

use crate::domain::notification::NotificationType;
use crate::services::notification_service::{NotificationService, NotificationServiceImpl};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct SendNotificationRequest {
    #[serde(rename = "notificationType")]
    pub notification_type: String,
    #[serde(rename = "recipientId")]
    pub recipient_id: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct SendNotificationResponse {
    pub success: bool,
    pub message: String,
}

pub async fn send_notification(
    req: web::Json<SendNotificationRequest>,
    notification_service: web::Data<NotificationServiceImpl>,
) -> impl Responder {
    tracing::info!(
        "Sending notification: type={}, recipient={}, subject={}",
        req.notification_type,
        req.recipient_id,
        req.subject
    );

    let notification_type = match req.notification_type.as_str() {
        "ApprovalRequest" => NotificationType::ApprovalRequest,
        "ApprovalCompleted" => NotificationType::ApprovalCompleted,
        "ApprovalRejected" => NotificationType::ApprovalRejected,
        "WorkflowCompleted" => NotificationType::WorkflowCompleted,
        _ => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "INVALID_TYPE".to_string(),
                message: format!("不正な通知タイプです: {}", req.notification_type),
            });
        }
    };

    match notification_service
        .send_notification(
            notification_type,
            &req.recipient_id,
            &req.subject,
            &req.body,
        )
        .await
    {
        Ok(_) => HttpResponse::Ok().json(SendNotificationResponse {
            success: true,
            message: "通知を送信しました".to_string(),
        }),
        Err(e) => {
            tracing::error!("Error sending notification: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: "サーバーエラーが発生しました".to_string(),
            })
        }
    }
}

pub async fn get_notification_history(
    query: web::Query<std::collections::HashMap<String, String>>,
    notification_service: web::Data<NotificationServiceImpl>,
) -> impl Responder {
    let recipient_id = match query.get("recipient_id") {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "MISSING_PARAMETER".to_string(),
                message: "recipient_idパラメータが必要です".to_string(),
            });
        }
    };

    tracing::info!("Getting notification history for recipient: {}", recipient_id);

    match notification_service.get_notification_history(recipient_id).await {
        Ok(notifications) => HttpResponse::Ok().json(notifications),
        Err(e) => {
            tracing::error!("Error getting notification history: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: "サーバーエラーが発生しました".to_string(),
            })
        }
    }
}

