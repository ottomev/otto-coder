# Server Diagnostic Commands for WebAssist Integration

Run these commands on **coder.otto.lk** to diagnose the WebAssist integration issue.

## 1. Check if WebAssist Config File Exists

```bash
# Check if config file exists
ls -la ~/.config/vibe-kanban/web-assist.toml

# If not there, check the assets directory
ls -la ~/otto-coder/config/web-assist.toml

# Or find it
find ~ -name "web-assist.toml" 2>/dev/null
```

## 2. View Current Configuration (if file exists)

```bash
cat ~/.config/vibe-kanban/web-assist.toml
```

**What to check:**
- `enabled = true` (is WebAssist enabled?)
- `webhook_secret = "..."` (is secret configured?)
- `[supabase]` section with `url`, `anon_key`, and `service_role_key`
- `projects_directory = "..."` (where projects are stored)

## 3. Check Otto Coder Database for WebAssist Projects

```bash
sqlite3 ~/.local/share/vibe-kanban/db.sqlite "SELECT * FROM web_assist_projects;"
```

**Expected result:**
- If no rows: No projects have been created via webhook
- If has rows: Projects exist (but maybe not syncing correctly)

## 4. Check Recent Logs for Webhook Activity

```bash
# Check for any webhook-related logs
journalctl --user -u otto-coder -n 200 --no-pager | grep -i "webhook"

# Check for WebAssist initialization logs
journalctl --user -u otto-coder -n 200 --no-pager | grep -i "webassist"

# Check for errors
journalctl --user -u otto-coder -p err -n 50 --no-pager
```

**What to look for:**
- `✅ "WebAssist integration initialized"` - Good!
- `❌ "WebAssist not enabled"` or `❌ "WebAssist configuration is incomplete"` - Config issue
- `✅ "Received webhook event: project.created"` - Webhook was received
- `❌ "Invalid webhook signature"` - Secret mismatch
- `❌ No webhook logs at all` - Webhook never triggered

## 5. Check Otto Coder Service Status

```bash
systemctl --user status otto-coder
```

**Expected:** `Active: active (running)`

## 6. Check if Webhook Endpoint is Accessible

```bash
# From the server, test the webhook endpoint
curl -I http://127.0.0.1:8080/api/web-assist/webhook
```

**Expected:** `HTTP/1.1 200 OK` or `HTTP/1.1 500` (not 404)

---

## What to Share

After running these commands, please share:

1. **Config file contents** (redact secrets if needed)
2. **Database query results** (how many projects exist?)
3. **Any relevant log entries** (especially errors or webhook-related)
4. **Service status** (is it running?)

This will help determine if the issue is:
- ❌ Config missing/incorrect
- ❌ WebAssist not enabled
- ❌ Webhook secret mismatch
- ❌ Service not running
- ❌ Database permissions
- ❌ Supabase webhook not configured

---

## Quick Copy-Paste Diagnostic Script

```bash
#!/bin/bash
echo "=== WebAssist Configuration ==="
cat ~/.config/vibe-kanban/web-assist.toml 2>/dev/null || echo "Config file not found"

echo -e "\n=== WebAssist Projects in Database ==="
sqlite3 ~/.local/share/vibe-kanban/db.sqlite "SELECT COUNT(*) as count FROM web_assist_projects;" 2>/dev/null || echo "Database query failed"

echo -e "\n=== Recent Webhook Logs ==="
journalctl --user -u otto-coder -n 100 --no-pager | grep -i "webhook\|webassist" | tail -20

echo -e "\n=== Recent Error Logs ==="
journalctl --user -u otto-coder -p err -n 10 --no-pager

echo -e "\n=== Service Status ==="
systemctl --user status otto-coder | head -10
```

Save this as `diagnose_webassist.sh`, make it executable (`chmod +x`), and run it.
