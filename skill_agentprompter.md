# Agent Prompter Skill

*Keeps projects progressing by managing prompts for next actions*

---

## Overview

This skill manages project momentum through a `prompt.md` file that contains actionable next steps. Agents must read this file before completing any task to ensure continuous progress.

---

## Core Philosophy

- **Never leave a project stagnant** - always write next steps to prompt.md
- **Always check prompt.md first** - before reporting "job done", read current prompts
- **Chain actions** - each completion should trigger the next logical step
- **Regular monitoring** - check project state and TODO regularly to keep things moving

---

## prompt.md Structure

Create `prompt.md` in project root with this format:

```markdown
# Project Prompts

## Current Focus
[Brief description of what's being worked on]

## Active Tasks
- [ ] Task 1 - Description
- [ ] Task 2 - Description

## Blockers
- None / [Description]

## Next Steps (Priority Order)
1. **HIGH**: Critical action needed
2. **MEDIUM**: Next logical step  
3. **LOW**: Future improvement

## Notes
[Any context for next agent]
```

---

## Subroutines

### 1. Check Prompts Before Completion
```
When about to report "job done":
1. Read {PROJECT_ROOT}/prompt.md
2. Check if any active tasks remain
3. If tasks exist, complete them or re-prioritize
4. Update prompt.md with new state
5. Only then report completion
```

### 2. Update Prompts After Work
```
After completing any substantial work:
1. Read current prompt.md
2. Remove completed tasks
3. Add any newly discovered tasks
4. Re-prioritize remaining work
5. Write updated prompt.md
```

### 3. Create Initial Prompts
```
When starting on a new project:
1. Read project files to understand scope
2. Identify immediate next steps
3. Create prompt.md with initial structure
4. Set priorities based on dependencies
```

### 4. Handle Blockers
```
When blocked on something:
1. Document blocker in prompt.md
2. Note what's needed to unblock
3. Suggest alternative paths if possible
4. Mark related tasks as blocked
```

### 5. Regular State Check
```
At the start of each session or when triggered:
1. Read {PROJECT_ROOT}/prompt.md
2. Read {PROJECT_ROOT}/TODO.md
3. Compare active items in both files
4. Sync any changes or new items found in TODO
5. Clean up stale prompts (older than 7 days)
6. Archive completed/obsolete prompts to prompt_archive/
```

### 6. Cleanup Routine
```
Run periodically or when prompts accumulate:
1. Count active items in prompt.md
2. If > 10 items, re-prioritize - move LOW to "parking lot"
3. Archive fully completed prompt sections to prompt_archive/
4. Create new prompt.md with clean state if needed
5. Keep only HIGH and MEDIUM priority items active
```

---

## Agent Integration

All agents must follow this workflow:

```
START → Regular State Check → Read prompt.md → Execute work → Update prompt.md → END
                                         ↓
                              If tasks remain:
                              → Continue or escalate
```

### Session Flow
```
1. Start session
2. Run "Regular State Check" subroutine
3. Read prompt.md for active tasks
4. Read TODO.md to sync any new items
5. Execute assigned work
6. Update prompt.md with results
7. Run "Cleanup Routine" if significant progress made
8. Report completion (only after steps 1-7)
```

---

## Naming Convention

Files using this skill:
- `prompt.md` - Active prompt file (in project root)
- `prompt_archive/` - Directory for old prompts
- `TODO.md` - Project todo list (synced with prompts)

---

## Quick Reference

| Trigger | Action |
|---------|--------|
| Start new task | Read prompt.md, run state check |
| About to finish | Read prompt.md, check TODO, update status |
| Complete work | Update prompt.md, sync with TODO |
| Get blocked | Document in prompt.md |
| Find new work | Add to prompt.md AND TODO.md |
| Session start | Run Regular State Check |
| Many tasks | Run Cleanup Routine |

---

*Last Updated: 2026-04-02*