#!/bin/bash

# APIテスト用スクリプト
# 環境変数で設定可能
BASE_URL=${BASE_URL:-http://localhost:8003}
API_BASE="${BASE_URL}/api/v1"
DATABASE_URL=${DATABASE_URL:-postgresql://gameday_user:gameday_password@localhost:5432/gameday_workflow_notification}


# 色の定義
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# テスト結果を記録
PASSED=0
FAILED=0

# テスト関数
test_endpoint() {
    local name=$1
    local method=$2
    local url=$3
    local data=$4
    local expected_status=$5

    echo -e "\n${YELLOW}テスト: ${name}${NC}"
    echo "  URL: ${url}"
    echo "  メソッド: ${method}"

    if [ "$method" = "POST" ] && [ -n "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST "${url}" \
            -H "Content-Type: application/json" \
            -d "${data}")
    else
        response=$(curl -s -w "\n%{http_code}" -X "${method}" "${url}")
    fi

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')

    if [ "$http_code" = "$expected_status" ]; then
        echo -e "  ${GREEN}✓ 成功${NC} (HTTP ${http_code})"
        echo "  レスポンス: ${body}" | head -c 200
        echo ""
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "  ${RED}✗ 失敗${NC} (期待: HTTP ${expected_status}, 実際: HTTP ${http_code})"
        echo "  レスポンス: ${body}"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

# ヘルスチェック（存在する場合）
echo "=== ヘルスチェック ==="
test_endpoint "ヘルスチェック" "GET" "${API_BASE}/health" "" "200" || true

# 承認フローテスト用のヘルパー関数
# ワークフローを開始して、各ステップで承認を実行し、current_stepが正しく更新されることを確認
test_workflow_approval_flow() {
    local app_type=$1
    local app_id=$2
    local flow_name=$3
    local step1_approver=$4
    local step2_approver=$5
    local step3_approver=$6  # オプション（3ステップの場合のみ）
    local amount=$7  # オプション（経費申請を経費精算に切り上げる金額判定に使用）

    echo -e "\n${YELLOW}=== ${flow_name} 承認フローテスト ===${NC}"

    # ワークフロー開始
    if [ -n "$amount" ]; then
        start_body="{
            \"applicationId\": \"${app_id}\",
            \"applicationType\": \"${app_type}\",
            \"companyId\": 1,
            \"amount\": ${amount}
        }"
    else
        start_body="{
            \"applicationId\": \"${app_id}\",
            \"applicationType\": \"${app_type}\",
            \"companyId\": 1
        }"
    fi
    test_endpoint \
        "ワークフロー開始 - ${flow_name}" \
        "POST" \
        "${API_BASE}/workflows/start" \
        "${start_body}" \
        "200"
    
    # ステップ1の承認実行とcurrent_step確認
    if [ -n "$step1_approver" ]; then
        echo -e "\n${YELLOW}ステップ1: 承認実行${NC}"
        response=$(curl -s -w "\n%{http_code}" -X POST "${API_BASE}/workflows/approve" \
            -H "Content-Type: application/json" \
            -d "{
                \"approvalId\": \"${app_id}-1\",
                \"applicationId\": \"${app_id}\",
                \"approverId\": \"${step1_approver}\",
                \"status\": \"approved\"
            }")
        
        http_code=$(echo "$response" | tail -n1)
        body=$(echo "$response" | sed '$d')
        
        if [ "$http_code" = "200" ]; then
            # jqが利用可能な場合はそれを使う、なければgrepで簡易パース
            if command -v jq >/dev/null 2>&1; then
                current_step=$(echo "$body" | jq -r '.currentStep // empty')
            else
                current_step=$(echo "$body" | grep -o '"currentStep":[0-9]*' | grep -o '[0-9]*' | head -1)
            fi
            echo -e "  ${GREEN}✓ ステップ1承認成功${NC} (HTTP ${http_code})"
            echo "  レスポンス: ${body}"
            if [ "$current_step" = "2" ]; then
                echo -e "  ${GREEN}✓ current_stepが正しく2に更新されました${NC}"
                PASSED=$((PASSED + 1))
            else
                echo -e "  ${RED}✗ current_stepが期待値(2)と異なります: ${current_step}${NC}"
                FAILED=$((FAILED + 1))
            fi
            PASSED=$((PASSED + 1))
        else
            echo -e "  ${RED}✗ ステップ1承認失敗${NC} (HTTP ${http_code})"
            echo "  レスポンス: ${body}"
            FAILED=$((FAILED + 1))
        fi
    fi
    
    # ステップ2の承認実行とcurrent_step確認
    if [ -n "$step2_approver" ]; then
        echo -e "\n${YELLOW}ステップ2: 承認実行${NC}"
        response=$(curl -s -w "\n%{http_code}" -X POST "${API_BASE}/workflows/approve" \
            -H "Content-Type: application/json" \
            -d "{
                \"approvalId\": \"${app_id}-2\",
                \"applicationId\": \"${app_id}\",
                \"approverId\": \"${step2_approver}\",
                \"status\": \"approved\"
            }")
        
        http_code=$(echo "$response" | tail -n1)
        body=$(echo "$response" | sed '$d')
        
        if [ "$http_code" = "200" ]; then
            # jqが利用可能な場合はそれを使う、なければgrepで簡易パース
            if command -v jq >/dev/null 2>&1; then
                current_step=$(echo "$body" | jq -r '.currentStep // empty')
                status=$(echo "$body" | jq -r '.status // empty')
            else
                current_step=$(echo "$body" | grep -o '"currentStep":[0-9]*' | grep -o '[0-9]*' | head -1)
                status=$(echo "$body" | grep -o '"status":"[^"]*"' | grep -o '"[^"]*"' | tr -d '"' | head -1)
            fi
            echo -e "  ${GREEN}✓ ステップ2承認成功${NC} (HTTP ${http_code})"
            echo "  レスポンス: ${body}"
            if [ -n "$step3_approver" ]; then
                # 3ステップの場合、ステップ2承認後はcurrent_step=3になる
                if [ "$current_step" = "3" ]; then
                    echo -e "  ${GREEN}✓ current_stepが正しく3に更新されました${NC}"
                    PASSED=$((PASSED + 1))
                else
                    echo -e "  ${RED}✗ current_stepが期待値(3)と異なります: ${current_step}${NC}"
                    FAILED=$((FAILED + 1))
                fi
            else
                # 2ステップの場合、ステップ2承認後は完了（status=completed）
                if [ "$status" = "completed" ]; then
                    echo -e "  ${GREEN}✓ ワークフローが完了しました（status=completed）${NC}"
                    PASSED=$((PASSED + 1))
                else
                    echo -e "  ${RED}✗ ワークフローのステータスが期待値(completed)と異なります: ${status}${NC}"
                    FAILED=$((FAILED + 1))
                fi
            fi
            PASSED=$((PASSED + 1))
        else
            echo -e "  ${RED}✗ ステップ2承認失敗${NC} (HTTP ${http_code})"
            echo "  レスポンス: ${body}"
            FAILED=$((FAILED + 1))
        fi
    fi
    
    # ステップ3の承認実行とcurrent_step確認（3ステップの場合のみ）
    if [ -n "$step3_approver" ]; then
        echo -e "\n${YELLOW}ステップ3: 承認実行${NC}"
        response=$(curl -s -w "\n%{http_code}" -X POST "${API_BASE}/workflows/approve" \
            -H "Content-Type: application/json" \
            -d "{
                \"approvalId\": \"${app_id}-3\",
                \"applicationId\": \"${app_id}\",
                \"approverId\": \"${step3_approver}\",
                \"status\": \"approved\"
            }")
        
        http_code=$(echo "$response" | tail -n1)
        body=$(echo "$response" | sed '$d')
        
        if [ "$http_code" = "200" ]; then
            # jqが利用可能な場合はそれを使う、なければgrepで簡易パース
            if command -v jq >/dev/null 2>&1; then
                current_step=$(echo "$body" | jq -r '.currentStep // empty')
                status=$(echo "$body" | jq -r '.status // empty')
            else
                current_step=$(echo "$body" | grep -o '"currentStep":[0-9]*' | grep -o '[0-9]*' | head -1)
                status=$(echo "$body" | grep -o '"status":"[^"]*"' | grep -o '"[^"]*"' | tr -d '"' | head -1)
            fi
            echo -e "  ${GREEN}✓ ステップ3承認成功${NC} (HTTP ${http_code})"
            echo "  レスポンス: ${body}"
            if [ "$status" = "completed" ]; then
                echo -e "  ${GREEN}✓ ワークフローが完了しました（status=completed）${NC}"
                PASSED=$((PASSED + 1))
            else
                echo -e "  ${RED}✗ ワークフローのステータスが期待値(completed)と異なります: ${status}${NC}"
                FAILED=$((FAILED + 1))
            fi
            PASSED=$((PASSED + 1))
        else
            echo -e "  ${RED}✗ ステップ3承認失敗${NC} (HTTP ${http_code})"
            echo "  レスポンス: ${body}"
            FAILED=$((FAILED + 1))
        fi
    fi
}

# 承認フローテスト
# 承認者ID（CompanyId=1の場合）:
# - エンジニア: 20001（申請者として使用）
# - 上長: 21051
# - 本部長: 1051
# - 経理: 16051

# 1. 出張申請: エンジニア申請 → 上長承認 → 本部長最終承認（3ステップ）
test_workflow_approval_flow "BusinessTrip" "bt-001" "出張申請" "20001" "21051" "1051"

# 2. 経費申請（通常）: エンジニア申請 → 上長承認（2ステップ、1段階承認）
test_workflow_approval_flow "Expense" "exp-001" "経費申請（通常）" "20001" "21051" ""

# 2b. 経費精算（高額・海外出張後など）: エンジニア申請 → 上長承認 → 経理承認
#     （3ステップ、2段階承認）。applicationType="Expense"のまま金額が閾値以上のため、
#     経費精算（ExpenseSettlement）のワークフロー定義に自動的に切り上げられる。
test_workflow_approval_flow "Expense" "exp-settlement-001" "経費精算（高額・2段階承認）" "20001" "21051" "16051" "150000"

# 3. 休暇申請: エンジニア申請 → 上長承認（2ステップ）
test_workflow_approval_flow "Vacation" "vac-001" "休暇申請" "20001" "21051" ""

# 4. プロモーション申請: 上長申請 → 本部長承認（2ステップ）
test_workflow_approval_flow "Promotion" "pro-001" "プロモーション申請" "21051" "1051" ""

# 承認バリデーションAPIテスト（異常系）
echo -e "\n=== 承認バリデーションAPIテスト（異常系） ==="

# 異常系: 不正なステータス
test_endpoint \
    "承認バリデーション - 不正なステータス" \
    "POST" \
    "${API_BASE}/workflows/validate-approval" \
    '{
        "approvalId": "test-1",
        "applicationId": "bt-001",
        "approverId": "21051",
        "status": "invalid"
    }' \
    "400"

# 異常系: 必須パラメータ不足
test_endpoint \
    "承認バリデーション - 必須パラメータ不足" \
    "POST" \
    "${API_BASE}/workflows/validate-approval" \
    '{
        "approvalId": "test-1"
    }' \
    "400"

# 通知履歴APIテスト
# 注意: 通知はワークフロー進行時に自動的に送信されます
echo -e "\n=== 通知履歴APIテスト ==="

# 正常系: 通知履歴取得
test_endpoint \
    "通知履歴取得 - 正常系" \
    "GET" \
    "${API_BASE}/notifications/history?recipient_id=21051" \
    "" \
    "200"

# 異常系: recipient_idパラメータ不足
test_endpoint \
    "通知履歴取得 - パラメータ不足" \
    "GET" \
    "${API_BASE}/notifications/history" \
    "" \
    "400"

# 結果サマリー
echo -e "\n=== テスト結果サマリー ==="
echo -e "${GREEN}成功: ${PASSED}${NC}"
echo -e "${RED}失敗: ${FAILED}${NC}"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}すべてのテストが成功しました！${NC}"
    exit 0
else
    echo -e "\n${RED}一部のテストが失敗しました。${NC}"
    exit 1
fi

