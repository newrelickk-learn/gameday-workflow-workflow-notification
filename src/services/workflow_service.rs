use crate::domain::notification::NotificationType;
use crate::domain::workflow::{ApprovalValidation, WorkflowInstance, WorkflowStatus, WorkflowStep};
use crate::infrastructure::db::WorkflowRepository;
use crate::services::notification_service::NotificationService;
use anyhow::Result;

// 経費精算（2段階承認）に切り上げる金額の閾値（円）。
// 海外出張後の経費精算のような高額な経費（航空券・宿泊費など）はこの金額以上になる想定で、
// 直属マネージャー承認に加えて経理部承認が必要になる。
// この閾値未満の通常の経費申請は、これまで通り1段階承認（マネージャーのみ）のままにする。
const EXPENSE_SETTLEMENT_AMOUNT_THRESHOLD: f64 = 100_000.0;

// 経費精算（2段階承認: 上長 → 経理）用のワークフロー定義名。
// 通常の経費申請（Expense、1段階承認）とは別のワークフロー定義として管理する。
const EXPENSE_SETTLEMENT_APPLICATION_TYPE: &str = "ExpenseSettlement";

#[async_trait::async_trait]
pub trait WorkflowService: Send + Sync {
    async fn validate_approval(
        &self,
        approval_id: &str,
        application_id: &str,
        approver_id: &str,
        status: &str,
    ) -> Result<ApprovalValidation>;

    async fn get_workflow_instance(
        &self,
        application_id: &str,
    ) -> Result<Option<WorkflowInstance>>;

    async fn start_workflow(
        &self,
        application_id: &str,
        application_type: &str,
        company_id: Option<i32>,
        applicant_id: Option<&str>,
        amount: Option<f64>,
    ) -> Result<(String, i32)>; // (workflow_instance_id, total_steps)

    async fn update_workflow_step(
        &self,
        application_id: &str,
        step: i32,
        status: WorkflowStatus,
    ) -> Result<()>;

    async fn get_workflow_definition(
        &self,
        application_type: &str,
        company_id: Option<i32>,
        amount: Option<f64>,
    ) -> Result<Vec<WorkflowStep>>;
}

pub struct WorkflowServiceImpl {
    repository: WorkflowRepository,
    notification_service: Box<dyn NotificationService>,
}

impl WorkflowServiceImpl {
    pub fn new(repository: WorkflowRepository, notification_service: Box<dyn NotificationService>) -> Self {
        Self { repository, notification_service }
    }

    // デモ用: ロールとCompanyIdから承認者IDを取得（実際の実装では、ユーザーサービスから取得）
    fn get_approver_id_by_role_and_company(&self, role: &str, company_id: i32) -> Option<String> {
        // CompanyIdに基づいて承認者IDを計算
        // 各会社ごとに異なるユーザーIDを使用
        match role {
            "上長" => {
                // 上長: ID 21051-21100 (各会社ごとに1名)
                // CompanyId 1 -> 21051, CompanyId 2 -> 21052, ...
                Some(format!("{}", 21051 + company_id - 1))
            },
            "本部長" | "開発本部長" => {
                // 本部長: ID 1051-1100 (各会社ごとに1名)
                // CompanyId 1 -> 1051, CompanyId 2 -> 1052, ...
                Some(format!("{}", 1051 + company_id - 1))
            },
            "経理" => {
                // 経理: ID 16051-16100 (各会社ごとに1名)
                // CompanyId 1 -> 16051, CompanyId 2 -> 16052, ...
                Some(format!("{}", 16051 + company_id - 1))
            },
            _ => None,
        }
    }

    // 経費申請（Expense）のうち、金額が閾値（EXPENSE_SETTLEMENT_AMOUNT_THRESHOLD）以上の場合は
    // 経費精算（ExpenseSettlement）のワークフロー定義（直属マネージャー承認 → 経理部承認の
    // 2段階承認）を使う。閾値未満の場合や経費申請以外の申請タイプの場合は、渡された
    // application_typeをそのまま使う（通常の経費申請は1段階承認のまま変わらない）。
    fn resolve_workflow_application_type(&self, application_type: &str, amount: Option<f64>) -> String {
        let is_expense = application_type.eq_ignore_ascii_case("expense");
        let exceeds_threshold = amount
            .map(|a| a >= EXPENSE_SETTLEMENT_AMOUNT_THRESHOLD)
            .unwrap_or(false);

        if is_expense && exceeds_threshold {
            EXPENSE_SETTLEMENT_APPLICATION_TYPE.to_string()
        } else {
            application_type.to_string()
        }
    }
}

#[async_trait::async_trait]
impl WorkflowService for WorkflowServiceImpl {
    async fn validate_approval(
        &self,
        approval_id: &str,
        application_id: &str,
        approver_id: &str,
        _status: &str,
    ) -> Result<ApprovalValidation> {
        // TODO: 実際の実装
        // 1. ワークフローインスタンスを取得
        // 2. 現在のステップを確認
        // 3. 承認者の権限を確認
        // 4. 次のステップを判定

        // スタブ実装
        let workflow_instance = self
            .get_workflow_instance(application_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Workflow instance not found"))?;

        let current_step = workflow_instance.current_step;
        let total_steps = self
            .repository
            .get_total_steps(workflow_instance.workflow_definition_id)
            .await
            .unwrap_or(1);
        let is_final_step = current_step >= total_steps;
        let next_step = if is_final_step {
            None
        } else {
            Some(current_step + 1)
        };

        Ok(ApprovalValidation {
            approval_id: approval_id.to_string(),
            application_id: application_id.to_string(),
            approver_id: approver_id.to_string(),
            current_step,
            is_final_step,
            next_step,
        })
    }

    async fn get_workflow_instance(
        &self,
        application_id: &str,
    ) -> Result<Option<WorkflowInstance>> {
        self.repository
            .get_workflow_instance_by_application_id(application_id)
            .await
    }

    async fn start_workflow(
        &self,
        application_id: &str,
        application_type: &str,
        company_id: Option<i32>,
        applicant_id: Option<&str>,
        amount: Option<f64>,
    ) -> Result<(String, i32)> {
        // CompanyIdが指定されていない場合は1をデフォルトとして使用
        let company_id = company_id.unwrap_or(1);

        // 経費申請（Expense）は金額基準で、経費精算（ExpenseSettlement、2段階承認）の
        // ワークフロー定義に切り上げるかどうかを判定する
        let resolved_application_type = self.resolve_workflow_application_type(application_type, amount);

        // ワークフロー定義を取得（CompanyIdを考慮）
        let workflow_definition_id = self
            .repository
            .get_workflow_definition_by_application_type_and_company_id(&resolved_application_type, company_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Workflow definition not found for application type: {} and company_id: {}", resolved_application_type, company_id))?;

        // ワークフローインスタンスを作成
        let instance = self
            .repository
            .create_workflow_instance(application_id, workflow_definition_id, 1)
            .await?;

        // 承認が必要なステップ（step 2以降）の承認者に通知を送信
        let steps = self.repository.get_workflow_steps(workflow_definition_id).await?;
        // step 1は申請者自身なので、step 2以降の承認者に通知を送信
        for step in steps.iter() {
            // step 1（エンジニア）は申請者自身なのでスキップ
            if step.step_number == 1 {
                continue;
            }
            // 承認が必要なステップ（step 2以降）の承認者に通知を送信
            if let Some(approver_id) = self.get_approver_id_by_role_and_company(&step.approver_role, company_id) {
                let _ = self.notification_service.send_notification(
                    NotificationType::ApprovalRequest,
                    &approver_id,
                    &format!("承認依頼: 申請ID {}", application_id),
                    &format!("申請ID {} の承認をお願いします（ステップ {}: {}）", application_id, step.step_number, step.approver_role),
                ).await;
            }
        }

        // 申請者に申請受付通知を送信（デモ用: applicant_idが提供された場合のみ）
        if let Some(applicant) = applicant_id {
            let _ = self.notification_service.send_notification(
                NotificationType::ApprovalRequest,
                applicant,
                &format!("申請を受け付けました: 申請ID {}", application_id),
                &format!("申請ID {} を受け付けました。承認プロセスが開始されました。", application_id),
            ).await;
        }

        // TODO: Kafkaイベントを発行

        // total_stepsを取得
        let total_steps = self.repository.get_total_steps(workflow_definition_id).await?;

        Ok((instance.id.to_string(), total_steps))
    }

    async fn update_workflow_step(
        &self,
        application_id: &str,
        step: i32,
        status: WorkflowStatus,
    ) -> Result<()> {
        let status_str = match status {
            WorkflowStatus::Pending => "pending",
            WorkflowStatus::InProgress => "in_progress",
            WorkflowStatus::Completed => "completed",
            WorkflowStatus::Rejected => "rejected",
            WorkflowStatus::Error => "error",
        };

        let workflow_instance = self
            .get_workflow_instance(application_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Workflow instance not found"))?;

        let total_steps = self
            .repository
            .get_total_steps(workflow_instance.workflow_definition_id)
            .await?;

        let is_final_step = step >= total_steps;
        let is_completed = matches!(status, WorkflowStatus::Completed);

        // ワークフローステップを更新
        self.repository
            .update_workflow_step(application_id, step, status_str)
            .await?;

        // 承認完了時、次のステップに進むか、最終承認完了の通知を送信
        if is_completed {
            if is_final_step {
                // 最終承認完了: 申請者に通知
                // デモ用: application_idから申請者IDを推測（実際の実装では、申請データから取得）
                let applicant_id = application_id; // デモ用: application_idを申請者IDとして使用
                let _ = self.notification_service.send_notification(
                    NotificationType::WorkflowCompleted,
                    applicant_id,
                    &format!("承認が完了しました: 申請ID {}", application_id),
                    &format!("申請ID {} のすべての承認が完了しました。", application_id),
                ).await;
            } else {
                // 次のステップの承認者に通知
                // TODO: ワークフローインスタンスからCompanyIdを取得する必要がある
                // 現在はデフォルトで1を使用（後で改善が必要）
                let company_id = 1; // TODO: ワークフローインスタンスから取得
                let steps = self.repository.get_workflow_steps(workflow_instance.workflow_definition_id).await?;
                let next_step_num = step + 1;
                if let Some(next_step) = steps.iter().find(|s| s.step_number == next_step_num) {
                    if let Some(approver_id) = self.get_approver_id_by_role_and_company(&next_step.approver_role, company_id) {
                        let _ = self.notification_service.send_notification(
                            NotificationType::ApprovalRequest,
                            &approver_id,
                            &format!("承認依頼: 申請ID {}", application_id),
                            &format!("申請ID {} の承認をお願いします（ステップ {}: {}）", application_id, next_step.step_number, next_step.approver_role),
                        ).await;
                    }
                }
            }
        }

        Ok(())
    }
    
    async fn get_workflow_definition(
        &self,
        application_type: &str,
        company_id: Option<i32>,
        amount: Option<f64>,
    ) -> Result<Vec<WorkflowStep>> {
        let company_id = company_id.unwrap_or(1);

        // start_workflowと同じ金額基準で、経費精算（ExpenseSettlement）の
        // ワークフロー定義を参照するかどうかを判定する
        let resolved_application_type = self.resolve_workflow_application_type(application_type, amount);

        // ワークフロー定義を取得
        let workflow_definition_id = self
            .repository
            .get_workflow_definition_by_application_type_and_company_id(&resolved_application_type, company_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Workflow definition not found for application type: {} and company_id: {}", resolved_application_type, company_id))?;

        // ワークフローステップを取得
        let steps = self.repository.get_workflow_steps(workflow_definition_id).await?;
        
        Ok(steps)
    }
}

