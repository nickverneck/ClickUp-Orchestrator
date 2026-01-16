# Workflow Editor Implementation

## Overview
- Added a workflow graph model persisted in the backend.
- Introduced workflow API endpoints for loading, saving, and start/pause state.
- Built a new frontend workflow editor with draggable nodes, edge actions, and per-node settings.
- Kept the existing settings page intact until runtime integration is complete.

## Backend
### Data Model
- Table: `workflow_configs`
- Columns:
  - `id` (PK)
  - `status` (string: `running` | `paused`)
  - `config` (JSON serialized)
  - `created_at`, `updated_at`

### Config Shape
```
{
  "version": 1,
  "nodes": [
    {
      "id": "node-queue",
      "type": "queue",
      "label": "Queue",
      "position": { "x": 120, "y": 180 },
      "settings": {
        "provider": "clickup",
        "baseBranch": "dev",
        "clickup": { "listId": "", "triggerStatus": "Ready for Dev" }
      }
    }
  ],
  "edges": [
    {
      "id": "edge-queue-dev",
      "source": "node-queue",
      "target": "node-dev",
      "output": null,
      "actions": [
        { "id": "action-create-branch", "type": "create_branch", "settings": { "branchPrefix": "task/" } }
      ]
    }
  ]
}
```

### Defaults
- Nodes: Queue, In Development (Agent), Completed, Failed, Stopped
- Edges:
  - Queue -> In Development (create branch)
  - In Development success -> Completed (update ClickUp status)
  - In Development error -> Failed (update ClickUp status)

### API
- `GET /api/workflow` -> `{ status, config }`
- `PUT /api/workflow` -> `{ config }`, returns `{ status, config }`
- `PUT /api/workflow/status` -> `{ status }`, returns `{ status, config }`
- `POST /api/workflow/start` -> `{ status, config }`
- `POST /api/workflow/pause` -> `{ status, config }`

## Frontend
- New route: `/workflow`
- Node types:
  - Queue node: provider (ClickUp/Jira/Custom), base branch, list id, trigger status
  - Agent node: model (codex/claude/gemini), system prompt, ClickUp status, capacity
  - Bucket node: completed/failed/stopped/custom
- Edge actions:
  - create branch
  - update ClickUp status
  - custom notes
- Editing rules:
  - Start/Pause button toggles workflow state.
  - Editing is disabled when workflow is running.
  - Save persists the workflow config.

## Follow-up Work
1. Connect workflow settings to the task runner and ClickUp poller.
2. Execute edge actions (branch creation, status updates) during transitions.
3. Enforce agent node constraints (single input, success/error outputs).
4. Add graph validation (cycles, unreachable nodes, missing required edges).
5. Implement per-node concurrency limits in the scheduler.
6. Expand provider support (Jira, custom sources) and integrate provider-specific settings.
7. Migrate existing settings into workflow defaults once runtime integration is complete.
