# WebAssist Integration Diagnostic Summary

**Date:** October 27, 2025
**Issue:** 3 projects in Supabase, but not appearing in Otto Coder

---

## Current Situation

### ✅ Projects in Supabase (Web Assist project: kbwbaudnxpvehhxdgkzb)

| Project Number | Company | Project ID | Status | Created |
|---------------|---------|------------|--------|---------|
| WA-2025-003 | Ottolabs | `50b25cca-7362-42b3-a1c4-a0e95c62ccf9` | active | Oct 27 12:24 |
| WA-2025-002 | Abra Foundations | `0c0520bd-ccdb-43aa-9de8-77867933545c` | active | Oct 27 08:46 |
| WA-2025-001 | ABRA Foundation | `b903f04f-b8d3-4f48-b651-1e5b2fe5ab12` | active | Oct 27 03:59 |

### ❓ Projects in Otto Coder
**Unknown** - Need to check with diagnostic script

---

## Possible Causes

### 1. Supabase Webhook Not Configured
The webhook that triggers Otto Coder when projects are created may not be set up.

**Check:**
- Supabase Dashboard → Database → Webhooks
- Should have a webhook for `projects` table INSERT events
- Should point to: `https://coder.otto.lk/api/web-assist/webhook`

### 2. Webhook Secret Mismatch
The HMAC signature verification might be failing.

**Check:**
- Webhook secret in Supabase webhook config
- Webhook secret in Otto Coder `config/web-assist.toml`
- They must match exactly

### 3. WebAssist Not Enabled in Otto Coder
The integration might be disabled in configuration.

**Check:**
- Otto Coder `config/web-assist.toml`
- `enabled = true`

### 4. Webhook Delivery Failures
Supabase might be unable to reach Otto Coder.

**Check:**
- Supabase webhook delivery logs
- Network connectivity
- SSL certificate issues

---

## Quick Diagnosis Steps

### Step 1: Run Status Check
```bash
cd /Users/web3relic/Documents/workspace/otto-coder
./scripts/check_webassist_status.sh
```

This will show:
- ✅ How many projects are in Otto Coder database
- ✅ Recent webhook activity in logs
- ✅ Configuration status
- ✅ Service health
- ✅ Any errors

### Step 2: Check Supabase Webhook Logs
1. Go to Supabase Dashboard
2. Navigate to Database → Webhooks
3. Find the webhook for `projects` table
4. Check "Delivery Logs" tab
5. Look for failed deliveries

### Step 3: Manually Trigger One Project
```bash
cd /Users/web3relic/Documents/workspace/otto-coder
./scripts/trigger_project_WA-2025-003.sh
```

This will:
- Send the `project.created` webhook for Ottolabs project
- Show you exactly what's being sent
- Display the response from Otto Coder
- Tell you if it succeeded or failed

---

## What Should Happen

When a project is created in Supabase:

```
┌─────────────┐         ┌──────────────┐         ┌──────────────┐
│  Supabase   │ webhook │ Otto Coder   │  write  │   Supabase   │
│  projects   ├────────→│  Backend     ├────────→│ otto_coder_* │
│   INSERT    │         │              │         │   tables     │
└─────────────┘         └──────────────┘         └──────────────┘
                              │
                              ↓
                        Creates:
                        - Otto project
                        - 9 tasks
                        - Next.js repo
```

**Expected results:**
1. New row in `web_assist_projects` table (Otto Coder local DB)
2. New Otto Coder project visible in dashboard
3. New row in `otto_coder_projects` (Supabase)
4. 9 rows in `otto_coder_tasks` (Supabase)
5. First task starts executing

---

## Manual Trigger Instructions

If webhook isn't working, you can manually trigger project creation:

```bash
# Trigger Ottolabs project (newest)
./scripts/trigger_project_WA-2025-003.sh

# For other projects, use the generic script:
./scripts/trigger_webhook.sh \
  "PROJECT_ID" \
  "WIZARD_ID" \
  "COMPANY_NAME" \
  "PROJECT_NUMBER" \
  "false"
```

**Example for WA-2025-002:**
```bash
./scripts/trigger_webhook.sh \
  "0c0520bd-ccdb-43aa-9de8-77867933545c" \
  "927179a9-dbb6-4538-8619-9fccfc357a26" \
  "Abra Foundations" \
  "WA-2025-002" \
  "false"
```

---

## Monitoring After Trigger

### Watch Otto Coder Logs
```bash
ssh ubuntu@coder.otto.lk 'journalctl --user -u otto-coder -f | grep -i webassist'
```

### Check Database
```bash
ssh ubuntu@coder.otto.lk 'sqlite3 ~/.local/share/vibe-kanban/db.sqlite "
  SELECT
    webassist_project_id,
    otto_project_id,
    current_stage,
    sync_status
  FROM web_assist_projects;
"'
```

### Check Supabase Tables
Use Supabase MCP or SQL Editor:
```sql
-- Check projects
SELECT * FROM otto_coder_projects;

-- Check tasks
SELECT stage_name, status, progress
FROM otto_coder_tasks
WHERE otto_project_id = '...'
ORDER BY stage_order;

-- Check deliverables
SELECT * FROM otto_coder_deliverables
WHERE otto_project_id = '...';
```

---

## Success Indicators

✅ **Webhook received:**
```
INFO Received webhook event: project.created
```

✅ **Project created:**
```
INFO Creating Otto Coder project for WebAssist project 50b25cca-... (Ottolabs)
```

✅ **Supabase sync:**
```
INFO Updated WebAssist task progress: project=50b25cca-..., stage=initial_review
```

✅ **First task started:**
```
INFO Starting first task (Initial Review): [task-id]
```

---

## Need Help?

1. **Run diagnostics:** `./scripts/check_webassist_status.sh`
2. **Check README:** `scripts/README_WEBASSIST.md`
3. **Review logs:** Look for errors in Otto Coder logs
4. **Test webhook:** Use manual trigger script
5. **Verify config:** Ensure all credentials are set

---

## Configuration Checklist

- [ ] WebAssist enabled in config (`enabled = true`)
- [ ] Webhook secret configured in Otto Coder
- [ ] Webhook secret configured in Supabase
- [ ] Supabase webhook created for projects table
- [ ] Webhook points to correct URL
- [ ] Supabase URL in Otto Coder config
- [ ] Supabase anon_key in Otto Coder config
- [ ] Supabase service_role_key in Otto Coder config
- [ ] Otto Coder service running
- [ ] Network connectivity between Supabase and Otto Coder

---

**Last Updated:** October 27, 2025
