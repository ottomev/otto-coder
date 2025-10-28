#!/bin/bash
#
# Trigger webhook for WA-2025-003 (Ottolabs)
# This script sends the project.created webhook to Otto Coder
#

set -e

OTTO_URL="https://coder.otto.lk/api/web-assist/webhook"

# Project data from Supabase
PROJECT_ID="50b25cca-7362-42b3-a1c4-a0e95c62ccf9"
WIZARD_ID="d4519e07-7d3f-4381-b0ee-7dab1594b330"
COMPANY="Ottolabs"
PROJECT_NUM="WA-2025-003"
RUSH="false"

# Build JSON payload
PAYLOAD=$(cat <<'EOF'
{
  "event": "project.created",
  "data": {
    "project_id": "50b25cca-7362-42b3-a1c4-a0e95c62ccf9",
    "project_number": "WA-2025-003",
    "company_name": "Ottolabs",
    "wizard_completion_id": "d4519e07-7d3f-4381-b0ee-7dab1594b330",
    "is_rush_delivery": false
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

# Read webhook secret from config file (if available)
CONFIG_FILE="$HOME/.config/vibe-kanban/web-assist.toml"
if [ -f "$CONFIG_FILE" ]; then
    WEBHOOK_SECRET=$(grep 'webhook_secret' "$CONFIG_FILE" | sed 's/.*=\s*"\(.*\)"/\1/' | tr -d ' ')
    if [ -n "$WEBHOOK_SECRET" ]; then
        echo "✅ Using webhook secret from config file"
    fi
else
    echo "⚠️  Config file not found at $CONFIG_FILE"
fi

# If no secret found, prompt user
if [ -z "$WEBHOOK_SECRET" ]; then
    echo
    echo "Please enter the webhook secret from config/web-assist.toml:"
    echo "(or press Ctrl+C to cancel and edit this script)"
    read -r WEBHOOK_SECRET
fi

if [ -z "$WEBHOOK_SECRET" ]; then
    echo "❌ ERROR: No webhook secret provided"
    exit 1
fi

# Compute HMAC signature
SIGNATURE=$(echo -n "$PAYLOAD" | openssl dgst -sha256 -hmac "$WEBHOOK_SECRET" | sed 's/^.* //')

echo "Signature: ${SIGNATURE:0:20}..."
echo

# Send webhook
echo "Sending webhook..."
HTTP_CODE=$(curl -s -w "%{http_code}" -o /tmp/webhook_response.txt \
  -X POST "$OTTO_URL" \
  -H "Content-Type: application/json" \
  -H "X-Supabase-Signature: $SIGNATURE" \
  -d "$PAYLOAD")

RESPONSE=$(cat /tmp/webhook_response.txt)

echo
echo "Response Code: $HTTP_CODE"
echo "Response Body: $RESPONSE"
echo

if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ Webhook sent successfully!"
    echo
    echo "Next steps:"
    echo "  1. Check Otto Coder logs:"
    echo "     ssh ubuntu@coder.otto.lk 'journalctl --user -u otto-coder -n 50 --no-pager | grep -i webassist'"
    echo
    echo "  2. Check database:"
    echo "     ssh ubuntu@coder.otto.lk 'sqlite3 ~/.local/share/vibe-kanban/db.sqlite \"SELECT * FROM web_assist_projects;\"'"
    echo
    echo "  3. Visit: https://coder.otto.lk/projects (look for WebAssist projects)"
else
    echo "❌ Webhook failed with HTTP $HTTP_CODE"
    echo
    echo "Common issues:"
    echo "  1. Webhook secret mismatch"
    echo "  2. WebAssist not enabled in Otto Coder config"
    echo "  3. Otto Coder service not running"
    echo
    echo "Check logs:"
    echo "  ssh ubuntu@coder.otto.lk 'journalctl --user -u otto-coder -n 100 --no-pager | tail -50'"
fi

rm -f /tmp/webhook_response.txt
