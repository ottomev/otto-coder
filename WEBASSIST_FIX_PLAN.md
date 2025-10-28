# WebAssist Integration Fix - Action Plan

**Date**: October 27, 2025
**Issue**: 3 projects in Supabase, but not appearing in Otto Coder

---

## Investigation Summary

### ✅ Code Review - All Good!

I've reviewed the entire codebase and confirmed:

1. **Webhook Handler** (`crates/web_assist/src/webhook.rs`):
   - ✅ Properly implements HMAC-SHA256 signature verification
   - ✅ Handles `project.created`, `approval.updated`, and `project.stage_changed` events
   - ✅ Delegates to ProjectManager correctly

2. **API Routes** (`crates/server/src/routes/web_assist.rs`):
   - ✅ `/api/web-assist/webhook` endpoint correctly configured
   - ✅ Extracts `X-Supabase-Signature` header
   - ✅ Returns 503 if WebAssist not initialized
   - ✅ `/api/web-assist/projects` endpoint queries local DB correctly

3. **Configuration System** (`crates/web_assist/src/config.rs`):
   - ✅ Proper TOML deserialization
   - ✅ Validates required fields (webhook_secret, Supabase credentials, etc.)
   - ✅ Clear error messages

4. **Initialization** (`crates/local-deployment/src/lib.rs`):
   - ✅ Loads config from `~/.local/share/vibe-kanban/web-assist.toml` (production)
   - ✅ Or from `dev_assets/web-assist.toml` (development)
   - ✅ Creates WebhookHandler, ProjectManager, and ApprovalSync components

### ❓ What's Missing?

The code is correct, so the issue is likely **configuration or Supabase webhook setup**.

---

## Root Cause Analysis

Projects exist in Supabase but not in Otto Coder because **the webhook was never triggered**. Possible reasons:

1. **Config file missing or incomplete** on coder.otto.lk
2. **WebAssist not enabled** (`enabled = false` in config)
3. **Webhook secret mismatch** (Supabase vs Otto Coder)
4. **Supabase webhook not configured** to trigger Otto Coder
5. **Otto Coder service not running** or crashed
6. **Projects created before webhook was set up**

---

## Action Plan

### Phase 1: Diagnose Server State

You need SSH access to `coder.otto.lk` to run these commands:

```bash
# 1. Check if config file exists
ls -la ~/.local/share/vibe-kanban/web-assist.toml

# 2. View config (if exists)
cat ~/.local/share/vibe-kanban/web-assist.toml

# 3. Check database for projects
sqlite3 ~/.local/share/vibe-kanban/db.sqlite "SELECT * FROM web_assist_projects;"

# 4. Check logs for webhook activity
journalctl --user -u otto-coder -n 200 --no-pager | grep -i "webhook\|webassist"

# 5. Check for errors
journalctl --user -u otto-coder -p err -n 50 --no-pager

# 6. Check service status
systemctl --user status otto-coder
```

**Or use the quick diagnostic script**:
See `SERVER_DIAGNOSTIC_COMMANDS.md` for a complete diagnostic script you can run.

---

### Phase 2: Create/Fix Configuration

If the config file is missing or incomplete, create it:

**Location**: `~/.local/share/vibe-kanban/web-assist.toml`

**Required fields**:
```toml
[web_assist]
enabled = true
webhook_secret = "your-secret-here"  # Generate with: openssl rand -hex 32
projects_directory = "/home/ubuntu/webassist-projects"

[web_assist.supabase]
url = "https://kbwbaudnxpvehhxdgkzb.supabase.co"
anon_key = "eyJ..."  # Get from Supabase dashboard
service_role_key = "eyJ..."  # Get from Supabase dashboard (keep secret!)

[web_assist.executor]
default_profile = "claude-sonnet-4-20250514"
```

**Full example**: See `config/web-assist.toml.example` for all options.

**After creating/updating config**:
```bash
systemctl --user restart otto-coder
```

---

### Phase 3: Configure Supabase Webhook

In Supabase Dashboard → Database → Webhooks:

1. **Create new webhook** (if not exists):
   - **Name**: Otto Coder Project Sync
   - **Table**: `projects`
   - **Events**: INSERT
   - **HTTP Request**:
     - **URL**: `https://coder.otto.lk/api/web-assist/webhook`
     - **Method**: POST
     - **HTTP Headers**:
       - Add header: `X-Supabase-Signature`
       - Value: Use the **same webhook_secret** from Otto Coder config

2. **Enable the webhook**

3. **Check delivery logs** for any failed deliveries

---

### Phase 4: Manually Trigger Existing Projects

Once config and webhook are set up, manually trigger the 3 existing projects:

```bash
# From your local machine (otto-coder directory)
cd /Users/web3relic/Documents/workspace/otto-coder

# Trigger Ottolabs project (WA-2025-003)
./scripts/trigger_project_WA-2025-003.sh

# If successful, trigger the other two
./scripts/trigger_webhook.sh \
  "0c0520bd-ccdb-43aa-9de8-77867933545c" \
  "927179a9-dbb6-4538-8619-9fccfc357a26" \
  "Abra Foundations" \
  "WA-2025-002" \
  "false"

./scripts/trigger_webhook.sh \
  "b903f04f-b8d3-4f48-b651-1e5b2fe5ab12" \
  "<wizard_id>" \
  "ABRA Foundation" \
  "WA-2025-001" \
  "false"
```

**Note**: You'll need the `wizard_completion_id` for WA-2025-001. Get it from Supabase:
```sql
SELECT id, wizard_completion_id FROM projects WHERE project_number = 'WA-2025-001';
```

---

### Phase 5: Verify Everything Works

After triggering projects, verify:

1. **Otto Coder Dashboard**: Visit https://coder.otto.lk/webassist
   - Should show all 3 projects
   - Should show company names and current stages

2. **Otto Coder Database**:
   ```bash
   ssh ubuntu@coder.otto.lk
   sqlite3 ~/.local/share/vibe-kanban/db.sqlite "SELECT * FROM web_assist_projects;"
   ```
   - Should have 3 rows

3. **Supabase Tables** (via Supabase MCP or SQL Editor):
   ```sql
   -- Should have 3 projects
   SELECT * FROM otto_coder_projects;

   -- Should have 27 tasks (9 per project)
   SELECT COUNT(*) FROM otto_coder_tasks;

   -- Check task progress
   SELECT
     p.webassist_project_id,
     t.stage_name,
     t.status,
     t.progress
   FROM otto_coder_projects p
   JOIN otto_coder_tasks t ON t.otto_project_id = p.otto_project_id
   ORDER BY p.webassist_project_id, t.stage_order;
   ```

4. **WebAssist Client Dashboard**:
   - Visit https://webassist.otto.lk/dashboard/[project-id] for each project
   - Should show "AI Agent Status" component
   - Should display current stage and progress

5. **Check Logs**:
   ```bash
   ssh ubuntu@coder.otto.lk
   journalctl --user -u otto-coder -n 100 --no-pager | grep -i webassist
   ```
   - Look for: "Creating Otto Coder project for WebAssist project..."
   - Look for: "Successfully synced task progress to Supabase..."

---

## Expected Flow (When Working Correctly)

```
┌─────────────────────────────────────────────────────────────────┐
│  User creates project in WebAssist (web-assist.otto.lk)        │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│  Supabase: INSERT into projects table                            │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│  Supabase Webhook: POST to coder.otto.lk/api/web-assist/webhook │
│  Headers: X-Supabase-Signature: <HMAC-SHA256>                   │
│  Body: { event: "project.created", data: {...} }                │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│  Otto Coder: WebhookHandler.handle_webhook()                    │
│  1. Verify HMAC signature                                        │
│  2. Parse webhook payload                                        │
│  3. Call ProjectManager.create_project_from_webhook()            │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│  Otto Coder: ProjectManager.create_project_from_webhook()       │
│  1. Create Otto project (in local DB)                            │
│  2. Create 9 tasks for stages                                    │
│  3. Create web_assist_projects record (local DB)                 │
│  4. Sync to Supabase: otto_coder_projects, otto_coder_tasks     │
│  5. Start first task (Initial Review)                            │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│  Supabase: Data in otto_coder_* tables                           │
│  - otto_coder_projects (1 row)                                   │
│  - otto_coder_tasks (9 rows)                                     │
│  - otto_coder_deliverables (grows as files are created)          │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│  WebAssist Frontend: Realtime subscription receives updates      │
│  useOttoCoderStatus hook fetches and displays progress           │
└─────────────────────────────────────────────────────────────────┘
```

---

## Quick Reference

### File Locations

**Local (your machine)**:
- Scripts: `/Users/web3relic/Documents/workspace/otto-coder/scripts/`
- Config example: `/Users/web3relic/Documents/workspace/otto-coder/config/web-assist.toml.example`
- Diagnostic docs: `/Users/web3relic/Documents/workspace/otto-coder/WEBASSIST_DIAGNOSTIC_SUMMARY.md`

**Server (coder.otto.lk)**:
- Config: `~/.local/share/vibe-kanban/web-assist.toml`
- Database: `~/.local/share/vibe-kanban/db.sqlite`
- Projects directory: `/home/ubuntu/webassist-projects/` (or as configured)
- Service: `systemctl --user status otto-coder`

**Supabase**:
- Project URL: `https://kbwbaudnxpvehhxdgkzb.supabase.co`
- Tables: `projects`, `otto_coder_projects`, `otto_coder_tasks`, `otto_coder_deliverables`
- Webhooks: Dashboard → Database → Webhooks

---

## Troubleshooting

### "No WebAssist Projects Yet" on coder.otto.lk/webassist

**Cause**: Otto Coder hasn't received any webhook events or config is disabled.

**Fix**:
1. Check config exists and `enabled = true`
2. Check logs for errors
3. Manually trigger a project

### Webhook Returns HTTP 503

**Cause**: WebAssist not initialized (config missing/invalid).

**Fix**: Create valid config file and restart service.

### Webhook Returns HTTP 500

**Cause**: Signature verification failed OR internal error.

**Fix**:
1. Check webhook secret matches in both places
2. Check logs for specific error

### Projects Appear in Otto Coder but Not in WebAssist Dashboard

**Cause**: Supabase sync failed (credentials missing/wrong).

**Fix**:
1. Add Supabase credentials to config
2. Restart Otto Coder
3. Check logs for Supabase API errors

### Tasks Not Progressing

**Cause**: Task executor not running or crashed.

**Fix**:
1. Check Otto Coder logs for executor errors
2. Check task status in database
3. Restart failed tasks from Otto Coder UI

---

## Success Indicators

When everything is working, you should see:

### In Logs:
```
INFO WebAssist integration initialized successfully
INFO Received webhook event: project.created
INFO Creating Otto Coder project for WebAssist project 50b25cca-... (Ottolabs)
INFO Successfully created Otto Coder project: [otto-project-id]
INFO Synced project status to Supabase: [webassist-project-id]
INFO Starting task: Initial Review
```

### In Dashboards:
- ✅ coder.otto.lk/webassist shows 3 projects
- ✅ webassist.otto.lk/dashboard/[id] shows "AI Agent Status"
- ✅ Current stage displays with progress percentage
- ✅ Task list shows which tasks are completed/in-progress

### In Database:
```sql
-- Should return 3
SELECT COUNT(*) FROM web_assist_projects;

-- Should return 3
SELECT COUNT(*) FROM otto_coder_projects;

-- Should return 27 (9 tasks × 3 projects)
SELECT COUNT(*) FROM otto_coder_tasks;
```

---

## Next Steps

1. **Run diagnostic commands** on coder.otto.lk (Phase 1)
2. **Share results** so we can determine the exact issue
3. **Fix configuration** based on diagnostic findings (Phase 2)
4. **Verify/configure Supabase webhook** (Phase 3)
5. **Manually trigger the 3 existing projects** (Phase 4)
6. **Test automatic webhook** by creating a new project (Phase 5)

---

**Need Help?**

1. Check `WEBASSIST_DIAGNOSTIC_SUMMARY.md` for detailed troubleshooting
2. Check `scripts/README_WEBASSIST.md` for script usage
3. Check `SERVER_DIAGNOSTIC_COMMANDS.md` for diagnostic commands
4. Review Otto Coder logs on the server

