# AgentManager Skill

*For managing AI agent workflows and job execution*

---

## Overview

The AgentManager coordinates AI agents to execute tasks from a project plan ([PROJECT_TRACKING_FILE]). It creates job sessions, assigns tasks, and ensures completion.

---

## Workflow

### 1. Read Project Plan
- Load `[PROJECT_TRACKING_FILE]` from the project root
- Parse tasks and their status (pending/in-progress/completed)
- Identify next pending task

### 2. Create Job Session
- Spawn a new AI agent session
- Provide context about the project
- Assign the pending task

### 3. Execute Task
- Prompt agent with task details
- Allow agent to work autonomously
- Monitor progress if possible

### 4. Verify Completion
- Check if task was completed successfully
- Update `[PROJECT_TRACKING_FILE]` with completion status
- Close the agent session

### 5. Proceed to Next Task
- Repeat until all tasks are complete
- Report overall progress

---

## Key Files

| File | Purpose |
|------|---------|
| `[PROJECT_TRACKING_FILE]` | Task list and status tracking |
| `[AGENT_CONFIG]` | Agent configuration |
| `skill.md` | Agent debugging guidelines |

---

## [PROJECT_TRACKING_FILE] Format

```markdown
# [PROJECT_NAME]

## Tasks

| # | Status | Description |
|---|--------|-------------|
| 1 | [ ] | Task description |
| 2 | [x] | Completed task |
| 3 | [ ] | Another task |

## Progress
- Completed: 1/3
- Pending: 2
```

---

## Agent Types

| Agent | Use Case |
|-------|----------|
| Explore | Research, analysis, code search |
| General | Multi-step tasks, implementation |

---

## Session Management

### Start Session
```bash
# Launch agent with task
task: "Research and implement feature X"
description: "Details about the task"
subagent_type: "explore"
```

### Monitor Progress
- Review agent output
- Check for errors
- Verify file changes

### Close Session
- Confirm task completion
- Document results
- Clean up resources

---

## Error Handling

If agent fails:
1. Log error details
2. Retry with different approach
3. Escalate if necessary
4. Mark task as failed in `[PROJECT_TRACKING_FILE]`

---

## Progress Reporting

After each task completion:
- Update `[PROJECT_TRACKING_FILE]` status
- Report: "Task N completed: [description]"
- Show remaining tasks

After all tasks:
- Report: "All tasks completed!"
- Summary of work done

---

## Integration

This skill works with:
- `[PROJECT_TRACKING_FILE]` for task tracking
- `[AGENT_CONFIG]` for agent configuration
- `skill.md` for debugging issues

---

*Last Updated: [date]*
