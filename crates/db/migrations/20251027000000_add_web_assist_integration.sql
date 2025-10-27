-- Migration: Add WebAssist Integration Tables
-- Links WebAssist projects (from Supabase) to Otto Coder projects
-- Manages approval workflows and stage synchronization

-- WebAssist project stages enum
CREATE TABLE IF NOT EXISTS web_assist_stage_enum (
    value TEXT PRIMARY KEY
);

INSERT INTO web_assist_stage_enum (value) VALUES
    ('initial_review'),
    ('ai_research'),
    ('design_mockup'),
    ('content_collection'),
    ('development'),
    ('quality_assurance'),
    ('client_preview'),
    ('deployment'),
    ('delivered');

-- Sync status enum
CREATE TABLE IF NOT EXISTS sync_status_enum (
    value TEXT PRIMARY KEY
);

INSERT INTO sync_status_enum (value) VALUES
    ('active'),
    ('paused'),
    ('error'),
    ('completed');

-- Approval status enum
CREATE TABLE IF NOT EXISTS approval_status_enum (
    value TEXT PRIMARY KEY
);

INSERT INTO approval_status_enum (value) VALUES
    ('pending'),
    ('approved'),
    ('rejected'),
    ('changes_requested');

-- Links WebAssist projects to Otto Coder projects
CREATE TABLE IF NOT EXISTS web_assist_projects (
    id TEXT PRIMARY KEY,
    webassist_project_id TEXT NOT NULL UNIQUE, -- From Supabase
    otto_project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    current_stage TEXT NOT NULL DEFAULT 'initial_review',
    stage_task_mapping TEXT NOT NULL, -- JSON: {"initial_review": "task_uuid", ...}
    sync_status TEXT NOT NULL DEFAULT 'active',
    last_synced_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (current_stage) REFERENCES web_assist_stage_enum(value),
    FOREIGN KEY (sync_status) REFERENCES sync_status_enum(value)
);

-- Tracks approval states across both systems
CREATE TABLE IF NOT EXISTS web_assist_approvals (
    id TEXT PRIMARY KEY,
    web_assist_project_id TEXT NOT NULL REFERENCES web_assist_projects(id) ON DELETE CASCADE,
    stage_name TEXT NOT NULL,
    approval_id TEXT, -- WebAssist approval ID from Supabase
    status TEXT NOT NULL DEFAULT 'pending',
    requested_at DATETIME NOT NULL,
    responded_at DATETIME,
    client_feedback TEXT,
    preview_url TEXT,
    deliverables TEXT NOT NULL DEFAULT '[]', -- JSON: [{name, url, type}]
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (stage_name) REFERENCES web_assist_stage_enum(value),
    FOREIGN KEY (status) REFERENCES approval_status_enum(value)
);

-- Indexes for fast lookups
CREATE INDEX IF NOT EXISTS idx_wa_projects_webassist_id ON web_assist_projects(webassist_project_id);
CREATE INDEX IF NOT EXISTS idx_wa_projects_otto_id ON web_assist_projects(otto_project_id);
CREATE INDEX IF NOT EXISTS idx_wa_approvals_project_stage ON web_assist_approvals(web_assist_project_id, stage_name);
CREATE INDEX IF NOT EXISTS idx_wa_approvals_status ON web_assist_approvals(status);

-- Trigger to update updated_at on web_assist_projects
CREATE TRIGGER IF NOT EXISTS update_web_assist_projects_timestamp
    AFTER UPDATE ON web_assist_projects
    FOR EACH ROW
    BEGIN
        UPDATE web_assist_projects SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

-- Trigger to update updated_at on web_assist_approvals
CREATE TRIGGER IF NOT EXISTS update_web_assist_approvals_timestamp
    AFTER UPDATE ON web_assist_approvals
    FOR EACH ROW
    BEGIN
        UPDATE web_assist_approvals SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;
