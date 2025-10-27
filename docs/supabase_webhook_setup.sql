-- =====================================================
-- Supabase Webhook Setup for Otto Coder Integration
-- =====================================================
--
-- This script sets up database triggers and functions to send
-- webhooks from Supabase to Otto Coder when WebAssist projects
-- are created or approvals are updated.
--
-- Prerequisites:
-- 1. Supabase HTTP extension must be enabled
-- 2. Replace OTTO_CODER_WEBHOOK_URL with your actual URL
-- 3. Replace WEBHOOK_SECRET with your HMAC secret key
--
-- =====================================================

-- Enable the HTTP extension if not already enabled
CREATE EXTENSION IF NOT EXISTS http;

-- =====================================================
-- Function: Send Webhook to Otto Coder
-- =====================================================

CREATE OR REPLACE FUNCTION send_otto_coder_webhook(
    event_name TEXT,
    payload JSONB
)
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    otto_url TEXT := 'https://otto-coder.your-domain.com/api/web-assist/webhook';
    webhook_secret TEXT := 'your-webhook-secret-here';
    payload_text TEXT;
    signature TEXT;
BEGIN
    -- Convert payload to text
    payload_text := payload::text;

    -- Calculate HMAC-SHA256 signature
    signature := encode(
        hmac(payload_text::bytea, webhook_secret::bytea, 'sha256'),
        'hex'
    );

    -- Send HTTP POST request
    PERFORM net.http_post(
        url := otto_url,
        headers := jsonb_build_object(
            'Content-Type', 'application/json',
            'X-Supabase-Signature', signature
        ),
        body := payload
    );

    -- Log the webhook (optional - for debugging)
    RAISE NOTICE 'Sent webhook: % to %', event_name, otto_url;

EXCEPTION WHEN OTHERS THEN
    -- Log error but don't fail the transaction
    RAISE WARNING 'Failed to send webhook: %', SQLERRM;
END;
$$;

-- =====================================================
-- Trigger Function: Project Created
-- =====================================================

CREATE OR REPLACE FUNCTION notify_otto_coder_project_created()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    webhook_payload JSONB;
BEGIN
    -- Build webhook payload
    webhook_payload := jsonb_build_object(
        'event', 'project.created',
        'project_id', NEW.id,
        'project_number', NEW.project_number,
        'company_name', NEW.company_name,
        'wizard_completion_id', NEW.wizard_completion_id,
        'is_rush_delivery', COALESCE(NEW.is_rush_delivery, false),
        'timestamp', NOW()
    );

    -- Send webhook asynchronously
    PERFORM send_otto_coder_webhook('project.created', webhook_payload);

    RETURN NEW;
END;
$$;

-- Create trigger on projects table
DROP TRIGGER IF EXISTS on_project_created ON projects;

CREATE TRIGGER on_project_created
    AFTER INSERT ON projects
    FOR EACH ROW
    EXECUTE FUNCTION notify_otto_coder_project_created();

-- =====================================================
-- Trigger Function: Approval Updated
-- =====================================================

CREATE OR REPLACE FUNCTION notify_otto_coder_approval_updated()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    webhook_payload JSONB;
BEGIN
    -- Only send webhook if status changed
    IF NEW.status != OLD.status THEN
        -- Build webhook payload
        webhook_payload := jsonb_build_object(
            'event', 'approval.updated',
            'approval_id', NEW.id,
            'project_id', NEW.project_id,
            'approval_type', NEW.approval_type,
            'status', NEW.status,
            'client_feedback', NEW.client_feedback,
            'responded_at', NEW.responded_at,
            'timestamp', NOW()
        );

        -- Send webhook asynchronously
        PERFORM send_otto_coder_webhook('approval.updated', webhook_payload);
    END IF;

    RETURN NEW;
END;
$$;

-- Create trigger on project_approvals table
DROP TRIGGER IF EXISTS on_approval_updated ON project_approvals;

CREATE TRIGGER on_approval_updated
    AFTER UPDATE ON project_approvals
    FOR EACH ROW
    EXECUTE FUNCTION notify_otto_coder_approval_updated();

-- =====================================================
-- Trigger Function: Stage Changed (Optional)
-- =====================================================
-- This is optional - only if you want to sync manual
-- stage changes made in WebAssist UI to Otto Coder

CREATE OR REPLACE FUNCTION notify_otto_coder_stage_changed()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    webhook_payload JSONB;
BEGIN
    -- Only send if stage status changed to 'in_progress'
    IF NEW.status = 'in_progress' AND OLD.status != 'in_progress' THEN
        webhook_payload := jsonb_build_object(
            'event', 'project.stage_changed',
            'project_id', NEW.project_id,
            'stage_id', NEW.id,
            'stage_name', NEW.stage_name,
            'stage_order', NEW.stage_order,
            'timestamp', NOW()
        );

        PERFORM send_otto_coder_webhook('project.stage_changed', webhook_payload);
    END IF;

    RETURN NEW;
END;
$$;

-- Create trigger on project_stages table (optional)
DROP TRIGGER IF EXISTS on_stage_changed ON project_stages;

CREATE TRIGGER on_stage_changed
    AFTER UPDATE ON project_stages
    FOR EACH ROW
    EXECUTE FUNCTION notify_otto_coder_stage_changed();

-- =====================================================
-- Verification & Testing
-- =====================================================

-- Test the webhook function
-- DO $$
-- DECLARE
--     test_payload JSONB;
-- BEGIN
--     test_payload := jsonb_build_object(
--         'event', 'test',
--         'message', 'This is a test webhook',
--         'timestamp', NOW()
--     );
--
--     PERFORM send_otto_coder_webhook('test', test_payload);
--     RAISE NOTICE 'Test webhook sent!';
-- END;
-- $$;

-- View webhook logs (if logging is enabled in your Supabase)
-- SELECT * FROM net._http_response ORDER BY created_at DESC LIMIT 10;

-- =====================================================
-- Cleanup (if needed)
-- =====================================================

-- To remove all triggers and functions:
-- DROP TRIGGER IF EXISTS on_project_created ON projects;
-- DROP TRIGGER IF EXISTS on_approval_updated ON project_approvals;
-- DROP TRIGGER IF EXISTS on_stage_changed ON project_stages;
-- DROP FUNCTION IF EXISTS notify_otto_coder_project_created();
-- DROP FUNCTION IF EXISTS notify_otto_coder_approval_updated();
-- DROP FUNCTION IF EXISTS notify_otto_coder_stage_changed();
-- DROP FUNCTION IF EXISTS send_otto_coder_webhook(TEXT, JSONB);

-- =====================================================
-- Important Notes
-- =====================================================
--
-- 1. SECURITY: Replace 'your-webhook-secret-here' with a strong random secret
--    Generate one using: openssl rand -hex 32
--
-- 2. URL: Replace 'https://otto-coder.your-domain.com' with your actual Otto Coder URL
--
-- 3. TESTING: After running this script, create a test project in WebAssist
--    and verify the webhook is received by Otto Coder
--
-- 4. MONITORING: Monitor Supabase logs for webhook failures
--    Check: Database -> Logs -> Postgres Logs
--
-- 5. RETRY LOGIC: If webhooks fail, they are logged but don't retry
--    Consider implementing retry logic or using Supabase Edge Functions
--
-- 6. PERFORMANCE: Webhooks are sent synchronously in the trigger
--    For high-volume systems, consider using a queue/job system
--
-- =====================================================
