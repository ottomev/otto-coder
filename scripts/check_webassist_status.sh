#!/bin/bash
#
# Check WebAssist Integration Status on coder.otto.lk
#

set -e

SERVER="coder.otto.lk"
SSH_USER="ubuntu"  # Adjust if different

echo "=========================================="
echo "WebAssist Integration Status Check"
echo "=========================================="
echo

# Check if we can SSH to the server
if ! ssh -q -o BatchMode=yes -o ConnectTimeout=5 "$SSH_USER@$SERVER" exit 2>/dev/null; then
    echo "⚠️  Cannot SSH to $SERVER"
    echo "Please ensure:"
    echo "  1. You have SSH access configured"
    echo "  2. SSH key is added to the server"
    echo "  3. User is '$SSH_USER' (adjust in script if different)"
    echo
    exit 1
fi

echo "✅ SSH connection OK"
echo

# Function to run command on remote server
remote_cmd() {
    ssh "$SSH_USER@$SERVER" "$@"
}

echo "1. Checking Otto Coder database for WebAssist projects..."
echo "---"
remote_cmd "sqlite3 ~/.local/share/vibe-kanban/db.sqlite \"SELECT COUNT(*) as project_count FROM web_assist_projects;\" 2>/dev/null || echo 'No projects found or database not accessible'"
echo

echo "2. Checking recent application logs for webhook activity..."
echo "---"
remote_cmd "journalctl --user -u otto-coder -n 100 --no-pager | grep -i 'webhook\|webassist' | tail -20 || echo 'No webhook-related logs found'"
echo

echo "3. Checking WebAssist configuration..."
echo "---"
remote_cmd "cat ~/.config/vibe-kanban/web-assist.toml 2>/dev/null || echo 'Config file not found at ~/.config/vibe-kanban/web-assist.toml'"
echo

echo "4. Checking if Otto Coder service is running..."
echo "---"
remote_cmd "systemctl --user status otto-coder | grep Active || echo 'Service status unavailable'"
echo

echo "5. Checking recent error logs..."
echo "---"
remote_cmd "journalctl --user -u otto-coder -p err -n 20 --no-pager || echo 'No error logs available'"
echo

echo "=========================================="
echo "Status Check Complete"
echo "=========================================="
echo
echo "If projects are in Supabase but not in Otto Coder:"
echo "  1. Check if webhook is configured in Supabase"
echo "  2. Verify webhook secret matches in both places"
echo "  3. Check Supabase webhook logs for delivery failures"
echo "  4. Use manual_webhook_trigger.py to test manually"
echo
