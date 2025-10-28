#!/bin/bash
#
# Manually trigger WebAssist webhook
# Usage: ./trigger_webhook.sh <project_id> <wizard_completion_id> <company_name> <project_number>
#

set -e

# Configuration
OTTO_URL="https://coder.otto.lk/api/web-assist/webhook"
WEBHOOK_SECRET=""  # TODO: Add your webhook secret from config/web-assist.toml

# Check arguments
if [ $# -lt 4 ]; then
    echo "Usage: $0 <project_id> <wizard_completion_id> <company_name> <project_number> [rush_delivery]"
    echo
    echo "Example:"
    echo "  $0 \\"
    echo "    '550e8400-e29b-41d4-a716-446655440000' \\"
    echo "    '660e8400-e29b-41d4-a716-446655440000' \\"
    echo "    'Acme Corp' \\"
    echo "    'WA-2025-001' \\"
    echo "    'false'"
    echo
    echo "To get project data from Supabase:"
    echo "  SELECT id, company_name, wizard_completion_id FROM projects WHERE ... ;"
    exit 1
fi

PROJECT_ID="$1"
WIZARD_ID="$2"
COMPANY="$3"
PROJECT_NUM="$4"
RUSH="${5:-false}"

# Check if webhook secret is set
if [ -z "$WEBHOOK_SECRET" ]; then
    echo "❌ ERROR: WEBHOOK_SECRET not set!"
    echo "Please edit this script and add the webhook secret from config/web-assist.toml"
    exit 1
fi

# Build JSON payload
PAYLOAD=$(cat <<EOF
{
  "event": "project.created",
  "data": {
    "project_id": "$PROJECT_ID",
    "project_number": "$PROJECT_NUM",
    "company_name": "$COMPANY",
    "wizard_completion_id": "$WIZARD_ID",
    "is_rush_delivery": $RUSH
  }
}
EOF
)

echo "=========================================="
echo "Triggering WebAssist Webhook"
echo "=========================================="
echo "Target: $OTTO_URL"
echo "Project: $COMPANY ($PROJECT_NUM)"
echo "Project ID: $PROJECT_ID"
echo

# Compute HMAC signature
SIGNATURE=$(echo -n "$PAYLOAD" | openssl dgst -sha256 -hmac "$WEBHOOK_SECRET" | sed 's/^.* //')

echo "Signature: ${SIGNATURE:0:20}..."
echo

# Send webhook
HTTP_CODE=$(curl -s -w "%{http_code}" -o /tmp/webhook_response.txt \
  -X POST "$OTTO_URL" \
  -H "Content-Type: application/json" \
  -H "X-Supabase-Signature: $SIGNATURE" \
  -d "$PAYLOAD")

RESPONSE=$(cat /tmp/webhook_response.txt)

echo "Response Code: $HTTP_CODE"
echo "Response Body: $RESPONSE"
echo

if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ Webhook sent successfully!"
    echo
    echo "Check Otto Coder database:"
    echo "  ssh ubuntu@coder.otto.lk"
    echo "  sqlite3 ~/.local/share/vibe-kanban/db.sqlite"
    echo "  SELECT * FROM web_assist_projects WHERE webassist_project_id='$PROJECT_ID';"
else
    echo "❌ Webhook failed with HTTP $HTTP_CODE"
    echo
    echo "Troubleshooting:"
    echo "  1. Check Otto Coder logs: journalctl --user -u otto-coder -n 50"
    echo "  2. Verify webhook secret matches config"
    echo "  3. Check if WebAssist is enabled in config"
fi

rm -f /tmp/webhook_response.txt
