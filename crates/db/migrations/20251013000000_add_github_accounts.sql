-- Add support for multiple GitHub accounts
-- Each project can be associated with a specific GitHub account

-- Create github_accounts table
CREATE TABLE github_accounts (
    id            BLOB PRIMARY KEY,
    username      TEXT NOT NULL UNIQUE,
    oauth_token   TEXT,
    pat           TEXT,
    primary_email TEXT,
    created_at    TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

-- Create index for faster username lookups
CREATE INDEX idx_github_accounts_username ON github_accounts(username);

-- Add github_account_id column to projects table
ALTER TABLE projects ADD COLUMN github_account_id BLOB REFERENCES github_accounts(id) ON DELETE SET NULL;

-- Create index for faster project-account lookups
CREATE INDEX idx_projects_github_account ON projects(github_account_id);
