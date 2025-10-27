# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

### Development
```bash
# Start development servers with hot reload (frontend + backend)
pnpm run dev

# Individual dev servers
npm run frontend:dev    # Frontend only (port 3000)
npm run backend:dev     # Backend only (port auto-assigned)

# Build production version
./build-npm-package.sh
```

### Testing & Validation
```bash
# Run all checks (frontend + backend)
npm run check

# Frontend specific
cd frontend && npm run lint          # Lint TypeScript/React code
cd frontend && npm run format:check  # Check formatting
cd frontend && npx tsc --noEmit     # TypeScript type checking

# Backend specific  
cargo test --workspace               # Run all Rust tests
cargo test -p <crate_name>          # Test specific crate
cargo test test_name                # Run specific test
cargo fmt --all -- --check          # Check Rust formatting
cargo clippy --all --all-targets --all-features -- -D warnings  # Linting

# Type generation (after modifying Rust types)
npm run generate-types               # Regenerate TypeScript types from Rust
npm run generate-types:check        # Verify types are up to date
```

### Database Operations
```bash
# SQLx migrations
sqlx migrate run                     # Apply migrations
sqlx database create                 # Create database

# Database is auto-copied from dev_assets_seed/ on dev server start
```

## Architecture Overview

### Tech Stack
- **Backend**: Rust with Axum web framework, Tokio async runtime, SQLx for database
- **Frontend**: React 18 + TypeScript + Vite, Tailwind CSS, shadcn/ui components  
- **Database**: SQLite with SQLx migrations
- **Type Sharing**: ts-rs generates TypeScript types from Rust structs
- **MCP Server**: Built-in Model Context Protocol server for AI agent integration

### Project Structure
```
crates/
├── server/         # Axum HTTP server, API routes, MCP server
├── db/            # Database models, migrations, SQLx queries
├── executors/     # AI coding agent integrations (Claude, Gemini, etc.)
├── services/      # Business logic, GitHub, auth, git operations
├── local-deployment/  # Local deployment logic
└── utils/         # Shared utilities

frontend/          # React application
├── src/
│   ├── components/  # React components (TaskCard, ProjectCard, etc.)
│   ├── pages/      # Route pages
│   ├── hooks/      # Custom React hooks (useEventSourceManager, etc.)
│   └── lib/        # API client, utilities

shared/types.ts    # Auto-generated TypeScript types from Rust
```

### Key Architectural Patterns

1. **Event Streaming**: Server-Sent Events (SSE) for real-time updates
   - Process logs stream to frontend via `/api/events/processes/:id/logs`
   - Task diffs stream via `/api/events/task-attempts/:id/diff`

2. **Git Worktree Management**: Each task execution gets isolated git worktree
   - Managed by `WorktreeManager` service
   - Automatic cleanup of orphaned worktrees

3. **Executor Pattern**: Pluggable AI agent executors
   - Each executor (Claude, Gemini, etc.) implements common interface
   - Actions: `coding_agent_initial`, `coding_agent_follow_up`, `script`

4. **MCP Integration**: Vibe Kanban acts as MCP server
   - Tools: `list_projects`, `list_tasks`, `create_task`, `update_task`, etc.
   - AI agents can manage tasks via MCP protocol

### API Patterns

- REST endpoints under `/api/*`
- Frontend dev server proxies to backend (configured in vite.config.ts)
- Authentication via GitHub OAuth (device flow)
- All database queries in `crates/db/src/models/`

### Development Workflow

1. **Backend changes first**: When modifying both frontend and backend, start with backend
2. **Type generation**: Run `npm run generate-types` after modifying Rust types
3. **Database migrations**: Create in `crates/db/migrations/`, apply with `sqlx migrate run`
4. **Component patterns**: Follow existing patterns in `frontend/src/components/`

### Testing Strategy

- **Unit tests**: Colocated with code in each crate
- **Integration tests**: In `tests/` directory of relevant crates  
- **Frontend tests**: TypeScript compilation and linting only
- **CI/CD**: GitHub Actions workflow in `.github/workflows/test.yml`

### Environment Variables

Build-time (set when building):
- `GITHUB_CLIENT_ID`: GitHub OAuth app ID (default: Bloop AI's app)
- `POSTHOG_API_KEY`: Analytics key (optional)

Runtime:
- `BACKEND_PORT`: Backend server port (default: auto-assign)
- `FRONTEND_PORT`: Frontend dev port (default: 3000)
- `HOST`: Backend host (default: 127.0.0.1)
- `DISABLE_WORKTREE_ORPHAN_CLEANUP`: Debug flag for worktrees

## Frontend Changes Deployment

**CRITICAL**: After making ANY changes to frontend code (components, styles, etc.), you MUST rebuild the frontend:

```bash
cd frontend && pnpm build
```

The production server serves static files from `frontend/dist` on port 8080. Without rebuilding, your changes will NOT be visible to the user.

## WebAssist Integration

### Overview

Otto Coder integrates with the WebAssist project tracking system to provide AI-powered website development. When a new WebAssist project is created, Otto Coder automatically:

1. Creates an Otto Coder project
2. Initializes a Next.js application
3. Creates 9 tasks (one per WebAssist stage)
4. Executes stages with AI agents (research, design, development, etc.)
5. Syncs progress back to WebAssist in real-time
6. Handles client approvals bidirectionally

### Architecture

```
crates/
└── web_assist/         # WebAssist integration crate
    ├── src/
    │   ├── models.rs           # Data models (WebAssistStage, WebAssistProject, etc.)
    │   ├── supabase_client.rs  # Supabase API client
    │   ├── webhook.rs          # Webhook receiver from Supabase
    │   ├── project_manager.rs  # Project lifecycle orchestration
    │   ├── stage_executor.rs   # Stage execution logic
    │   └── approval_sync.rs    # Bidirectional approval sync
    └── prompts/                # Stage-specific AI prompts
        ├── 01_initial_review.md
        ├── 02_ai_research.md
        ├── 03_design_mockup.md
        ├── ... (9 total)
```

### WebAssist 9-Stage Workflow

| Stage | Duration | Type | Approval Required |
|-------|----------|------|-------------------|
| 1. Initial Review | 2h | Human | No |
| 2. AI Research | 2h | AI | No |
| 3. Design Mockup | 8h | AI | Yes |
| 4. Content Collection | 6h | AI | Yes |
| 5. Development | 16h | AI | No |
| 6. Quality Assurance | 4h | Human | No |
| 7. Client Preview | 6h | Human | Yes |
| 8. Deployment | 4h | AI | No |
| 9. Delivered | 0h | Complete | No |

### Configuration

Create `config/web-assist.toml`:

```toml
[web_assist]
enabled = true
webhook_secret = "your-webhook-secret"
projects_directory = "~/web-assist-projects"

[web_assist.supabase]
url = "https://your-project.supabase.co"
anon_key = "your-anon-key"
service_role_key = "your-service-role-key"

[web_assist.executor]
default_profile = "claude/claude-code"
research_timeout_minutes = 120
development_timeout_minutes = 960
```

See `config/web-assist.toml.example` for full configuration options.

### API Endpoints

WebAssist integration adds these endpoints:

- `POST /api/web-assist/webhook` - Receive webhooks from Supabase
- `GET /api/web-assist/projects/:id` - Get project status
- `GET /api/web-assist/projects/:id/stages/:stage/deliverables` - Get stage deliverables
- `POST /api/web-assist/approvals/:id` - Submit approval decision
- `GET /api/web-assist/projects/:id/approvals` - Get all approvals
- `POST /api/web-assist/projects/:id/sync` - Manual sync trigger

### Database Schema

New tables (see `crates/db/migrations/20251027000000_add_web_assist_integration.sql`):

- `web_assist_projects` - Links WebAssist projects to Otto projects
- `web_assist_approvals` - Tracks approval states
- Enum tables for stages, sync status, approval status

### Supabase Webhook Setup

Run `docs/supabase_webhook_setup.sql` in your Supabase SQL editor to set up:

1. Webhook function for Otto Coder
2. Trigger on `projects` table (new project created)
3. Trigger on `project_approvals` table (approval updated)
4. Trigger on `project_stages` table (stage changed - optional)

### Project File Structure

```
~/web-assist-projects/
└── {webassist_project_id}/
    ├── project/                    # Next.js app (git repo)
    │   ├── app/
    │   ├── components/
    │   ├── public/
    │   └── package.json
    ├── deliverables/               # Stage outputs
    │   ├── 01_initial_review/
    │   ├── 02_research/
    │   ├── 03_design/
    │   ├── 04_content/
    │   ├── 05_development/  (symlink to ../project)
    │   ├── 06_qa/
    │   ├── 07_preview/
    │   └── 08_deployment/
    └── .wa-project.json            # Metadata
```

### Development Workflow

1. **When WebAssist Creates Project**:
   - Supabase sends webhook to `/api/web-assist/webhook`
   - Otto Coder creates project and tasks
   - Initializes Next.js in `~/web-assist-projects/{id}/project/`
   - Starts first task (Initial Review)

2. **As Tasks Complete**:
   - Stage executor detects completion
   - If approval required: Creates approval request in both systems
   - If no approval: Auto-advances to next stage
   - Syncs progress to WebAssist Supabase

3. **When Client Approves** (from either UI):
   - Approval synced to both systems
   - Otto Coder resumes workflow
   - Starts next task

### Testing

```bash
# Test webhook locally
curl -X POST http://localhost:8080/api/web-assist/webhook \
  -H "Content-Type: application/json" \
  -d '{
    "event": "project.created",
    "project_id": "test-uuid",
    "company_name": "Test Company",
    "wizard_completion_id": "test-uuid",
    "is_rush_delivery": false
  }'

# Check project status
curl http://localhost:8080/api/web-assist/projects/{project_id}
```

### Frontend Team Documentation

Full API specification for WebAssist frontend team:
- See `docs/WEB_ASSIST_FRONTEND_REQUIREMENTS.md`

### Common Issues

1. **Webhook not received**: Check Supabase triggers and Otto Coder logs
2. **Project not created**: Verify webhook secret matches
3. **Approval not syncing**: Check Supabase service role key permissions
4. **Next.js init fails**: Ensure `npx` and Node.js are installed

### Related Files

- Configuration: `config/web-assist.toml.example`
- Database migration: `crates/db/migrations/20251027000000_add_web_assist_integration.sql`
- Supabase setup: `docs/supabase_webhook_setup.sql`
- Frontend API docs: `docs/WEB_ASSIST_FRONTEND_REQUIREMENTS.md`
- Prompt templates: `crates/web_assist/prompts/*.md`