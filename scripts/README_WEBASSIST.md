# WebAssist Integration Scripts

Scripts for testing and troubleshooting the WebAssist × Otto Coder integration.

## Available Scripts

### 1. `check_webassist_status.sh`
Checks the status of WebAssist integration on the remote server.

**Usage:**
```bash
./scripts/check_webassist_status.sh
```

**What it checks:**
- Otto Coder database for WebAssist projects
- Recent webhook activity in logs
- WebAssist configuration
- Service status
- Recent errors

### 2. `trigger_project_WA-2025-003.sh`
Manually triggers the `project.created` webhook for the Ottolabs project (WA-2025-003).

**Usage:**
```bash
./scripts/trigger_project_WA-2025-003.sh
```

**What it does:**
- Sends a `project.created` webhook to Otto Coder
- Uses actual project data from Supabase
- Computes proper HMAC signature
- Reports success/failure

**Requirements:**
- Webhook secret from `config/web-assist.toml`
- Network access to coder.otto.lk

### 3. `trigger_webhook.sh`
Generic webhook trigger for any project.

**Usage:**
```bash
./scripts/trigger_webhook.sh <project_id> <wizard_id> <company> <project_number> [rush]
```

**Example:**
```bash
./scripts/trigger_webhook.sh \
  "50b25cca-7362-42b3-a1c4-a0e95c62ccf9" \
  "d4519e07-7d3f-4381-b0ee-7dab1594b330" \
  "Ottolabs" \
  "WA-2025-003" \
  "false"
```

### 4. `manual_webhook_trigger.py`
Python script for webhook testing with more features.

**Usage:**
```bash
python3 scripts/manual_webhook_trigger.py
```

## Troubleshooting Workflow

### Projects not showing up in Otto Coder?

**Step 1: Check Supabase projects**
```bash
# You already have 3 projects in Supabase:
# - WA-2025-003 (Ottolabs)
# - WA-2025-002 (Abra Foundations)
# - WA-2025-001 (ABRA Foundation)
```

**Step 2: Check Otto Coder status**
```bash
./scripts/check_webassist_status.sh
```

This will show:
- How many WebAssist projects are in Otto Coder DB
- Recent webhook logs
- Configuration status
- Any errors

**Step 3: Check Supabase webhook configuration**

In Supabase Dashboard → Database → Webhooks:
- Check if webhook is configured for `projects` table
- Verify webhook URL: `https://coder.otto.lk/api/web-assist/webhook`
- Check if webhook is enabled
- Look at webhook delivery logs for failures

**Step 4: Manually trigger a project**
```bash
# Trigger the newest project (Ottolabs)
./scripts/trigger_project_WA-2025-003.sh
```

**Step 5: Check Otto Coder logs**
```bash
ssh ubuntu@coder.otto.lk 'journalctl --user -u otto-coder -n 100 --no-pager | grep -i webassist'
```

Look for:
- `"Received webhook event: project.created"`
- `"Creating Otto Coder project for WebAssist project"`
- `"Successfully created Otto Coder project"`
- Any errors

**Step 6: Verify in database**
```bash
ssh ubuntu@coder.otto.lk 'sqlite3 ~/.local/share/vibe-kanban/db.sqlite "SELECT * FROM web_assist_projects;"'
```

## Common Issues

### 1. Webhook signature verification fails
**Symptoms:** HTTP 500 error, logs show "Invalid webhook signature"

**Fix:**
- Ensure webhook secret matches in both places:
  - Supabase webhook configuration
  - Otto Coder `config/web-assist.toml`

### 2. WebAssist not enabled
**Symptoms:** Webhook returns HTTP 503 (Service Unavailable)

**Fix:**
- Edit `config/web-assist.toml` and set `enabled = true`
- Restart Otto Coder service

### 3. Missing Supabase credentials
**Symptoms:** Projects created but no sync to Supabase tables

**Fix:**
- Add Supabase URL, anon_key, and service_role_key to config
- Restart Otto Coder

### 4. Webhook not configured in Supabase
**Symptoms:** Projects in Supabase but never trigger Otto Coder

**Fix:**
- Go to Supabase Dashboard → Database → Webhooks
- Create webhook for `projects` table
- Set trigger: `INSERT`
- Set URL: `https://coder.otto.lk/api/web-assist/webhook`
- Add header: `X-Supabase-Signature` with HMAC key

## Configuration Example

**`config/web-assist.toml`:**
```toml
enabled = true
webhook_secret = "your-secret-key-here"
projects_directory = "/home/ubuntu/webassist-projects"

[supabase]
url = "https://kbwbaudnxpvehhxdgkzb.supabase.co"
anon_key = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
service_role_key = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."  # SECRET!

[executor]
default_profile = "claude-sonnet-4-20250514"
research_timeout_minutes = 120

[approvals]
require_human_review = false
```

## Next Steps After Successful Trigger

1. **Check dashboard**: Visit https://coder.otto.lk/projects
2. **Monitor progress**: Watch Supabase Realtime for updates in `otto_coder_tasks`
3. **Check Supabase tables**:
   - `otto_coder_projects` - Should have 1 row per project
   - `otto_coder_tasks` - Should have 9 rows per project
   - `otto_coder_deliverables` - Will populate as AI creates files

## Support

If issues persist:
1. Check Otto Coder logs thoroughly
2. Verify network connectivity between Supabase and Otto Coder
3. Test webhook endpoint directly with curl
4. Review Supabase webhook delivery logs
