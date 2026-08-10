use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tracing;

use crate::services::workflow_service::{WorkflowService, WorkflowServiceImpl};

#[derive(Debug, Deserialize)]
pub struct ValidateApprovalRequest {
    #[serde(rename = "approvalId")]
    pub approval_id: String,
    #[serde(rename = "applicationId")]
    pub application_id: String,
    #[serde(rename = "approverId")]
    pub approver_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateApprovalResponse {
    pub valid: bool,
    #[serde(rename = "currentStep")]
    pub current_step: i32,
    #[serde(rename = "isFinalStep")]
    pub is_final_step: bool,
    #[serde(rename = "nextStep", skip_serializing_if = "Option::is_none")]
    pub next_step: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

pub async fn validate_approval(
    req: web::Json<ValidateApprovalRequest>,
    workflow_service: web::Data<WorkflowServiceImpl>,
) -> impl Responder {
    tracing::info!(
        "Validating approval: approval_id={}, application_id={}, approver_id={}, status={}",
        req.approval_id,
        req.application_id,
        req.approver_id,
        req.status
    );

    match workflow_service
        .validate_approval(
            &req.approval_id,
            &req.application_id,
            &req.approver_id,
            &req.status,
        )
        .await
    {
        Ok(validation) => {
            // ステータスがapprovedまたはrejectedの場合のみ有効
            let valid = req.status == "approved" || req.status == "rejected";

            let response = ValidateApprovalResponse {
                valid,
                current_step: validation.current_step,
                is_final_step: validation.is_final_step,
                next_step: validation.next_step,
                message: if valid {
                    Some("承認可能です".to_string())
                } else {
                    Some("承認ステータスが不正です".to_string())
                },
            };

            if valid {
                HttpResponse::Ok().json(response)
            } else {
                HttpResponse::BadRequest().json(ErrorResponse {
                    error: "VALIDATION_ERROR".to_string(),
                    message: "承認ステータスが不正です".to_string(),
                })
            }
        }
        Err(e) => {
            tracing::error!("Error validating approval: {}", e);
            
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                HttpResponse::NotFound().json(ErrorResponse {
                    error: "NOT_FOUND".to_string(),
                    message: "指定された承認IDのワークフローが見つかりません".to_string(),
                })
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "INTERNAL_ERROR".to_string(),
                    message: "サーバーエラーが発生しました".to_string(),
                })
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct StartWorkflowRequest {
    #[serde(rename = "applicationId")]
    pub application_id: String,
    #[serde(rename = "applicationType")]
    pub application_type: String,
    #[serde(rename = "companyId")]
    pub company_id: Option<i32>,
    // 経費申請（Expense）の金額。一定額以上の場合は経費精算（ExpenseSettlement、
    // 2段階承認: 上長→経理）のワークフロー定義に切り上げるかどうかの判定に使う。
    pub amount: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct StartWorkflowResponse {
    #[serde(rename = "workflowInstanceId")]
    pub workflow_instance_id: String,
    #[serde(rename = "applicationId")]
    pub application_id: String,
    #[serde(rename = "currentStep")]
    pub current_step: i32,
    #[serde(rename = "totalSteps")]
    pub total_steps: i32,
    pub status: String,
}

pub async fn start_workflow(
    req: web::Json<StartWorkflowRequest>,
    workflow_service: web::Data<WorkflowServiceImpl>,
) -> impl Responder {
    tracing::info!(
        "Starting workflow: application_id={}, application_type={}",
        req.application_id,
        req.application_type
    );

    match workflow_service
        .start_workflow(&req.application_id, &req.application_type, req.company_id, None, req.amount)
        .await
    {
        Ok((workflow_instance_id, total_steps)) => {
            let response = StartWorkflowResponse {
                workflow_instance_id,
                application_id: req.application_id.clone(),
                current_step: 1,
                total_steps,
                status: "pending".to_string(),
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::error!("Error starting workflow: {}", e);
            
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                HttpResponse::NotFound().json(ErrorResponse {
                    error: "NOT_FOUND".to_string(),
                    message: format!("ワークフロー定義が見つかりません: {}", req.application_type),
                })
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "INTERNAL_ERROR".to_string(),
                    message: "サーバーエラーが発生しました".to_string(),
                })
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ApproveWorkflowRequest {
    #[serde(rename = "approvalId")]
    pub approval_id: String,
    #[serde(rename = "applicationId")]
    pub application_id: String,
    #[serde(rename = "approverId")]
    pub approver_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ApproveWorkflowResponse {
    #[serde(rename = "applicationId")]
    pub application_id: String,
    #[serde(rename = "currentStep")]
    pub current_step: i32,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub async fn approve_workflow(
    req: web::Json<ApproveWorkflowRequest>,
    workflow_service: web::Data<WorkflowServiceImpl>,
) -> impl Responder {
    use crate::domain::workflow::WorkflowStatus;

    tracing::info!(
        "Approving workflow: approval_id={}, application_id={}, approver_id={}, status={}",
        req.approval_id,
        req.application_id,
        req.approver_id,
        req.status
    );

    // まずバリデーション
    let validation = match workflow_service
        .validate_approval(
            &req.approval_id,
            &req.application_id,
            &req.approver_id,
            &req.status,
        )
        .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Error validating approval: {}", e);
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                return HttpResponse::NotFound().json(ErrorResponse {
                    error: "NOT_FOUND".to_string(),
                    message: "指定された承認IDのワークフローが見つかりません".to_string(),
                });
            } else {
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "INTERNAL_ERROR".to_string(),
                    message: "サーバーエラーが発生しました".to_string(),
                });
            }
        }
    };

    // ステータスがapprovedまたはrejectedの場合のみ処理
    if req.status != "approved" && req.status != "rejected" {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "VALIDATION_ERROR".to_string(),
            message: "承認ステータスが不正です".to_string(),
        });
    }

    // ワークフローインスタンスを取得
    let workflow_instance = match workflow_service
        .get_workflow_instance(&req.application_id)
        .await
    {
        Ok(Some(instance)) => instance,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: "ワークフローインスタンスが見つかりません".to_string(),
            });
        }
        Err(e) => {
            tracing::error!("Error getting workflow instance: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: "サーバーエラーが発生しました".to_string(),
            });
        }
    };

    let current_step = workflow_instance.current_step;
    let workflow_status = if req.status == "approved" {
        if validation.is_final_step {
            WorkflowStatus::Completed
        } else {
            WorkflowStatus::InProgress
        }
    } else {
        WorkflowStatus::Rejected
    };

    // ワークフローステップを更新
    let next_step = if req.status == "approved" && !validation.is_final_step {
        validation.next_step.unwrap_or(current_step + 1)
    } else {
        current_step
    };

    // レスポンス用のステータス文字列を先に作成
    let status_str = match workflow_status {
        WorkflowStatus::Completed => "completed".to_string(),
        WorkflowStatus::Rejected => "rejected".to_string(),
        WorkflowStatus::InProgress => "in_progress".to_string(),
        _ => "pending".to_string(),
    };

    match workflow_service
        .update_workflow_step(&req.application_id, next_step, workflow_status)
        .await
    {
        Ok(_) => {
            let response = ApproveWorkflowResponse {
                application_id: req.application_id.clone(),
                current_step: next_step,
                status: status_str,
                message: if req.status == "approved" {
                    if validation.is_final_step {
                        Some("すべての承認が完了しました".to_string())
                    } else {
                        Some(format!("ステップ{}の承認が完了しました。次のステップに進みます。", current_step))
                    }
                } else {
                    Some("承認が拒否されました".to_string())
                },
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::error!("Error updating workflow step: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: "サーバーエラーが発生しました".to_string(),
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetWorkflowDefinitionRequest {
    #[serde(rename = "applicationType")]
    pub application_type: String,
    #[serde(rename = "companyId")]
    pub company_id: Option<i32>,
    // start_workflowと同様、経費精算（ExpenseSettlement）判定に使う金額
    pub amount: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct WorkflowStepResponse {
    #[serde(rename = "stepNumber")]
    pub step_number: i32,
    #[serde(rename = "approverRole")]
    pub approver_role: String,
    #[serde(rename = "isRequired")]
    pub is_required: bool,
}

#[derive(Debug, Serialize)]
pub struct GetWorkflowDefinitionResponse {
    pub steps: Vec<WorkflowStepResponse>,
}

pub async fn get_workflow_definition(
    req: web::Query<GetWorkflowDefinitionRequest>,
    workflow_service: web::Data<WorkflowServiceImpl>,
) -> impl Responder {
    tracing::info!(
        "Getting workflow definition: application_type={}, company_id={:?}",
        req.application_type,
        req.company_id
    );

    match workflow_service
        .get_workflow_definition(&req.application_type, req.company_id, req.amount)
        .await
    {
        Ok(steps) => {
            let response = GetWorkflowDefinitionResponse {
                steps: steps
                    .into_iter()
                    .map(|s| WorkflowStepResponse {
                        step_number: s.step_number,
                        approver_role: s.approver_role,
                        is_required: s.is_required,
                    })
                    .collect(),
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::error!("Error getting workflow definition: {}", e);
            
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                HttpResponse::NotFound().json(ErrorResponse {
                    error: "NOT_FOUND".to_string(),
                    message: format!("ワークフロー定義が見つかりません: {}", req.application_type),
                })
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "INTERNAL_ERROR".to_string(),
                    message: "サーバーエラーが発生しました".to_string(),
                })
            }
        }
    }
}

