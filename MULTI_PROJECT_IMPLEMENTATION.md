# Multi-Project Support Implementation Guide

## Overview

This document describes the implementation of multi-project support for the ClickUp Orchestrator. The system now allows users to manage multiple independent projects, each with:
- Separate git repositories
- Independent ClickUp configurations
- Project-specific agent settings
- Isolated workflows and tasks

## Database Changes

### Migration: `m20260209_000001_add_projects`

**New Table: `projects`**
```sql
CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',  -- 'active' or 'archived'
    repo_path TEXT NOT NULL,                -- Absolute filesystem path
    github_url TEXT,                        -- GitHub clone URL (optional)
    dev_branch TEXT NOT NULL DEFAULT 'dev',
    clickup_workspace_id TEXT,
    clickup_space_id TEXT,
    clickup_folder_id TEXT,
    clickup_list_id TEXT,
    agent_prompt TEXT,
    agent_model TEXT DEFAULT 'claude',
    parallel_limit INTEGER DEFAULT 1,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
```

**Modified Tables:**
- `workflow_configs`: Added nullable `project_id` column
- `orchestrator_tasks`: Added nullable `project_id` column

**Migration Strategy:**
- On first run, migration reads global settings from `settings` table
- Creates "Default Project" with those settings
- Links all existing workflows and tasks to the default project
- Enables zero-downtime upgrade for existing installations

## Backend API Endpoints

### Projects Management

```
GET    /api/projects              - List all projects with stats
POST   /api/projects              - Create new project
GET    /api/projects/{id}         - Get project details
PUT    /api/projects/{id}         - Update project settings
DELETE /api/projects/{id}         - Delete project (cascade workflows/tasks)
POST   /api/projects/{id}/archive - Archive project
POST   /api/projects/clone        - Clone from GitHub
```

### Project-Scoped Workflows

```
GET    /api/projects/{id}/workflows  - List workflows for project
POST   /api/projects/{id}/workflows  - Create workflow in project
```

### Project-Scoped Tasks

```
GET    /api/tasks?project_id={id}    - Filter tasks by project
```

## ClickUp Poller Refactoring

The ClickUp poller now supports multi-project polling:

```rust
async fn poll_and_process(ctx: AppContext) {
    // Approach 1: Multi-project mode (when projects table is populated)
    let projects = projects::Entity::find()
        .filter(projects::Column::Status.eq("active"))
        .all(db)
        .await;

    for project in projects {
        // Poll each project's ClickUp list independently
        poll_and_process_project(db, project).await;
    }

    // Fallback: Legacy single-project mode (when no projects exist)
    poll_and_process_legacy(db).await;
}
```

**Key Features:**
- Automatic project detection
- Per-project parallel task limits
- Per-project ClickUp list configuration
- Per-project git repository isolation
- Backward compatible with legacy single-project setup
- Worktrees created under `{project.repo_path}/worktrees/{task_name}`

## Frontend Architecture

### API Client
**File:** `frontend/src/lib/api/projects.ts`

Provides complete CRUD operations:
- `listProjects()` - Fetch all projects with stats
- `getProject(id)` - Fetch single project
- `createProject(data)` - Create new project
- `updateProject(id, data)` - Update project
- `deleteProject(id)` - Delete project
- `archiveProject(id)` - Archive project
- `cloneProjectFromGithub(data)` - Clone from GitHub

### State Management
**File:** `frontend/src/lib/stores/project.svelte.ts`

Reactive store using Svelte 5:
- `projectStore` - Singleton instance
- `useCurrentProject()` - Hook for components
- Auto-persists to localStorage
- Reactive `currentProjectId`, `project`, `isLoading`, `error`

### Pages

#### Project Selection (`/projects`)
- Grid view of all projects
- Status badges (active/archived)
- Project stats (workflow count, task count)
- Empty state with CTA
- Click card to select and navigate

#### Project Creation (`/projects/new`)
Multi-step wizard:
1. **Step 1:** Choose creation type
   - Create from existing folder
   - Clone from GitHub
2. **Step 2:** Repository setup
   - Project name & description
   - Repository path/GitHub URL
   - Development branch
3. **Step 3:** Configuration
   - ClickUp list ID
   - Agent model & prompt
   - Parallel task limit

#### Project Dashboard (`/projects/{id}`)
- Project overview with key settings
- Repository information
- ClickUp configuration status
- Agent settings summary
- Quick links to workflows and settings

#### Workflows List (`/projects/{id}/workflows`)
- List workflows for specific project
- Create new workflow for project
- Click to edit workflow

#### Project Settings (`/projects/{id}/settings`)
- Edit all project fields
- Git configuration
- ClickUp settings
- Agent configuration
- Archive/delete options
- Danger zone with confirmations

## Implementation Checklist

### Backend ✅ COMPLETE
- [x] Database migration with auto-migration logic
- [x] Projects controller with CRUD endpoints
- [x] Project git service for repo operations
- [x] Workflow controller project scoping
- [x] Tasks controller project filtering
- [x] ClickUp poller multi-project support
- [x] Routes registration in app.rs

### Frontend ✅ COMPLETE
- [x] Projects API client
- [x] Project state store
- [x] Project selection page
- [x] Project creation wizard (3-step)
- [x] Project dashboard
- [x] Workflows list page
- [x] Project settings page

### Optional (Can be done in next phase)
- [ ] Sidebar project selector dropdown
- [ ] Update landing page to use projects page
- [ ] Extract reusable project components
- [ ] Add project templates
- [ ] Bulk project operations
- [ ] Project sharing/collaboration

## Testing Instructions

### 1. Run Database Migration

```bash
cd backend
cargo loco db migrate
```

**Verification:**
- Check that `projects` table exists
- Verify columns match schema
- Confirm indexes created
- Check that existing workflows linked to default project

### 2. Test Backend API

```bash
# List projects
curl http://localhost:5150/api/projects

# Create project
curl -X POST http://localhost:5150/api/projects \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Project",
    "repo_path": "/tmp/test-repo",
    "dev_branch": "dev"
  }'

# Get project
curl http://localhost:5150/api/projects/1

# Update project
curl -X PUT http://localhost:5150/api/projects/1 \
  -H "Content-Type: application/json" \
  -d '{"name": "Updated Name"}'

# Create workflow in project
curl -X POST http://localhost:5150/api/projects/1/workflows \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Workflow"}'

# List workflows for project
curl http://localhost:5150/api/projects/1/workflows

# List tasks for project
curl http://localhost:5150/api/tasks?project_id=1
```

### 3. Test Frontend Navigation

1. Navigate to `http://localhost:5173/projects`
   - Should see project selection page
   - Can click "New Project" button

2. Create new project
   - Step 1: Select "Create from Existing Folder"
   - Step 2: Enter project name and local repo path
   - Step 3: Configure agent and ClickUp settings
   - Should redirect to project dashboard

3. View project workflows
   - Click "New Workflow" button on dashboard
   - Should navigate to workflows page for project
   - Create new workflow should be project-scoped

4. Update project settings
   - Click "Settings" button
   - Update any field
   - Save changes
   - Verify API call succeeds

### 4. Test Multi-Project Isolation

1. Create 2 projects with different repos
2. Create workflows in each project
3. Verify:
   - Workflows appear in correct project
   - Tasks filtered by project_id
   - ClickUp poller processes each project separately
   - Worktrees created in correct project repo paths

### 5. Test ClickUp Poller

1. Configure first project with ClickUp list ID
2. Create task in ClickUp with "Ready for Dev" status
3. Wait for poller (30-second intervals)
4. Verify:
   - Task created in orchestrator_tasks with project_id
   - Worktree created in project's repo path
   - Agent spawned for task
   - Task logs contain project context

5. Configure second project and repeat
   - Verify both projects poll independently
   - Worktrees isolated per project

## Migration Path

### For Existing Installations

1. **Before Migration:**
   - Note current global settings (repo path, dev branch, ClickUp list, agent settings)
   - Backup database

2. **Run Migration:**
   - `cargo loco db migrate`
   - Migration auto-creates "Default Project" with current settings

3. **Verification:**
   - All existing workflows linked to default project
   - All existing tasks linked to default project
   - Global settings still accessible in settings table

4. **Gradual Adoption:**
   - Can use legacy single-project workflow
   - Create new projects as needed
   - Poller supports both old and new modes

### For New Installations

1. Application loads without projects configured
2. ClickUp poller skips (no projects to poll)
3. Create first project via UI
4. Poller automatically starts processing

## File Organization

```
backend/
  migration/src/
    m20260209_000001_add_projects.rs     [NEW]
  src/
    controllers/
      projects.rs                         [NEW]
      workflow.rs                         [MODIFIED]
      tasks.rs                            [MODIFIED]
    services/
      project_git.rs                      [NEW]
    initializers/
      clickup_poller.rs                   [MODIFIED]
    app.rs                                [MODIFIED]

frontend/
  src/
    lib/
      api/
        projects.ts                       [NEW]
      stores/
        project.svelte.ts                 [NEW]
    routes/
      projects/
        +page.svelte                      [NEW]
        new/
          +page.svelte                    [NEW]
        [id]/
          +page.svelte                    [NEW]
          workflows/
            +page.svelte                  [NEW]
          settings/
            +page.svelte                  [NEW]
```

## Backward Compatibility

### Legacy Mode
- If no projects table or no active projects found
- ClickUp poller falls back to reading global settings
- Existing workflows (project_id = NULL) still work
- API queries still support project_id = NULL

### API Compatibility
- All existing endpoints unchanged
- New project-scoped endpoints added as new routes
- Old endpoints continue working with NULL project_id

## Performance Considerations

### Database Indexes
- `idx_projects_status` - Quick lookup of active projects
- `idx_workflow_configs_project_id` - Filter workflows by project
- `idx_orchestrator_tasks_project_id` - Filter tasks by project

### Poller Optimization
- Processes projects sequentially (can be parallelized in future)
- Respects per-project parallel_limit
- Early exit if no active projects
- Maintains 30-second polling interval per poller cycle

## Future Enhancements

1. **Project Templates**
   - Pre-configured project templates
   - One-click project creation

2. **Project Grouping**
   - Organization/team support
   - Shared projects

3. **Advanced Filtering**
   - Search projects
   - Filter by status/agent/configuration

4. **Bulk Operations**
   - Archive multiple projects
   - Batch update settings

5. **Project Analytics**
   - Task completion rates per project
   - Agent performance per project

6. **Parallel Poller**
   - Process multiple projects concurrently
   - Per-project polling schedules

## Troubleshooting

### Migration Failed
- Check database permissions
- Verify sqlite3 CLI tools installed
- Check logs for SQL errors

### ClickUp Poller Not Running
- Verify at least one active project exists
- Check project has valid ClickUp list ID
- Check git repository path is accessible
- Review logs for project-specific errors

### Tasks Not Showing in Project
- Verify task.project_id matches project.id
- Check task created after project existed
- Check task status filtering

### Worktrees in Wrong Location
- Verify project.repo_path is correct
- Check git repository permissions
- Verify path exists

## Support & Questions

For questions or issues:
1. Check task logs in orchestrator_tasks table
2. Review ClickUp poller logs
3. Verify database migration completed
4. Check project configuration in UI
