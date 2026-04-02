# Agent Configuration

This file defines path exclusions and workspace boundaries for agents working on ascii-cam.

---

## Path Exclusions

Agents should NOT access files outside these paths:

### Research Agent
- `/home/crunch/ascii-cam/research/` - Read/Write
- `/home/crunch/ascii-cam/skill.md` - Read (reference for debug agent)
- `/home/crunch/ascii-cam/` - Read (analysis source)

**Excluded:**
- `*/.cargo/` - Build artifacts
- `*/target/` - Compiled binaries
- `/home/crunch/*` (except ascii-cam/)

### Debug Agent
- `/home/crunch/ascii-cam/` - Full access (source, docs, configs)
- `/home/crunch/ascii-cam/DEBUG/` - Read/Write (debug work)

**Excluded:**
- `*/.cargo/`
- `*/target/`
- `/home/crunch/*` (except ascii-cam/)

### Manager Agent
- `/home/crunch/ascii-cam/PROJECT.md` - Read/Write (task tracking)
- `/home/crunch/ascii-cam/.management/AgentManager.md` - Read (AgentManager workflow)
- `/home/crunch/ascii-cam/` - Read (project context)

**Excluded:**
- `*/.cargo/`
- `*/target/`
- `/home/crunch/*` (except ascii-cam/)

### Engineer Agent
- `/home/crunch/ascii-cam/research/` - Read/Write
- `/home/crunch/ascii-cam/PROJECT.md` - Read/Write (task tracking)
- `/home/crunch/ascii-cam/TODO.md` - Read/Write (task planning)
- `/home/crunch/ascii-cam/.management/` - Read/Write (workflow files)
- `/home/crunch/ascii-cam/AGENTS.md` - Read/Write (agent config)
- `/home/crunch/ascii-cam/` - Read (project context)

**Excluded:**
- `*/.cargo/` - Build artifacts
- `*/target/` - Compiled binaries
- `/home/crunch/*` (except ascii-cam/)

### AgentEngineer
- `/home/crunch/ascii-cam/.breedingpool/` - Read/Write (skill creation)
- `/home/crunch/ascii-cam/PROJECT.md` - Read (task requirements)
- `/home/crunch/ascii-cam/TODO.md` - Read (capability needs)
- `/home/crunch/ascii-cam/.management/` - Read (existing skill patterns)
- `/home/crunch/ascii-cam/AGENTS.md` - Read/Write (agent config updates)
- `/home/crunch/ascii-cam/research/` - Read (best practices)

**Excluded:**
- `*/.cargo/`
- `*/target/`
- `/home/crunch/*` (except ascii-cam/.breedingpool/)
- `/home/crunch/ascii-cam/src/` - Source code

### Analyst
- `/home/crunch/ascii-cam/.management/Analyst.md` - Read/Write (analysis skill)
- `/home/crunch/ascii-cam/PROJECT.md` - Read/Write (project overview)
- `/home/crunch/ascii-cam/TODO.md` - Read/Write (task planning)
- `/home/crunch/ascii-cam/USERFINDINGS.md` - Read (user issues)
- `/home/crunch/ascii-cam/research/` - Read/Write (market research)
- `/home/crunch/ascii-cam/.management/` - Read (workflow files)

**Excluded:**
- `*/.cargo/` - Build artifacts
- `*/target/` - Compiled binaries
- `/home/crunch/*` (except ascii-cam/)
- `/home/crunch/ascii-cam/src/` - Source code (focus on planning, not implementation)

---

## Workspace Boundaries

| Agent | Allowed Paths | Max Depth |
|-------|---------------|-----------|
| Research | `src/`, `research/`, `*.md` | 4 |
| Debug | `/home/crunch/ascii-cam/` (full) | 5 |
| Manager | `PROJECT.md`, `.management/`, `src/` | 4 |
| Engineer | `research/`, `PROJECT.md`, `TODO.md`, `.management/`, `AGENTS.md` | 4 |
| Analyst | `.management/`, `PROJECT.md`, `TODO.md`, `USERFINDINGS.md`, `research/` | 4 |
| AgentEngineer | `.breedingpool/`, `PROJECT.md` (read), `TODO.md` (read), `.management/` (read), `AGENTS.md` | 3 |

---

## USERFINDINGS Analysis Workflow

When `USERFINDINGS.md` exists in the project, agents should:

1. **Read and Analyze**: Review USERFINDINGS.md for documented issues
2. **Categorize**: Group findings by priority (Critical/Core/New)
3. **Create Analysis**: Write USERFINDINGS_ANALYSIS.md with solutions
4. **Update TODO**: Add checkboxes to TODO.md for each actionable item
5. **Update Architecture**: Modify SVG diagrams if needed
6. **Implement**: Work through prioritized checklist items

---

## Agent Prompter Integration

All agents **MUST** follow the AgentPrompter skill workflow:

1. **Read prompt.md first** - Before starting any work, check for active prompts
2. **Check before completion** - Before reporting "job done", read prompt.md
3. **Update after work** - After completing tasks, update prompt.md with new state
4. **Regular state check** - At session start, sync prompt.md with PROJECT.md/TODO.md
5. **Cleanup** - Keep prompts manageable, archive stale items

See `/home/crunch/ascii-cam/skill_agentprompter.md` for the complete skill specification.

---

## Quick Reference

```bash
# Valid paths for research agent
/home/crunch/ascii-cam/src/main.rs
/home/crunch/ascii-cam/research/analysis.md
/home/crunch/ascii-cam/Cargo.toml

# Valid paths for Engineer agent
/home/crunch/ascii-cam/TODO.md
/home/crunch/ascii-cam/PROJECT.md
/home/crunch/ascii-cam/.management/Engineer.md

# Invalid paths (should be rejected)
~/.cargo/registry/
/home/crunch/Downloads/
/home/crunch/.config/
```

---

*Last Updated: 2026-04-12*