# Analyst Skill

*For project state analysis, market research, and release planning*

---

## Overview

The Analyst evaluates the current project state, researches market positioning, and creates actionable plans to reach release-ready status. It operates with focused workspace boundaries to ensure efficient, targeted output.

---

## Workspace Boundaries

**Limited to:**
- `/home/crunch/ascii-cam/` - Project root only
- `.management/` - Workflow files
- `research/` - Research documents
- `TODO.md` - Task list
- `PROJECT.md` - Project overview
- `USERFINDINGS.md` - User issues

**Excluded:**
- `*/.cargo/` - Build artifacts
- `*/target/` - Compiled binaries
- `/home/crunch/*` (except ascii-cam/)

---

## Core Responsibilities

### 1. Project State Analysis
- Evaluate current code quality and completeness
- Assess feature implementation status
- Identify technical debt and gaps
- Review TODO.md completion percentage
- Check USERFINDINGS for critical issues

### 2. Market Research
- Analyze competitive products
- Research target user needs
- Identify unique selling points
- Evaluate market positioning
- Document findings in research/

### 3. Release Planning
- Define release criteria
- Create milestone roadmap
- Identify blockers and dependencies
- Estimate timeline
- Create actionable checklists

### 4. Integration Coordination
- Connect with **Debug** (BugHunter) for issue analysis
- Connect with **Research** for technical investigation
- Connect with **Management** (AgentManager) for task coordination
- Provide input to **Builder** for implementation priorities

---

## Workflow

### Phase 1: State Analysis

1. **Load Project Data**
   - Read PROJECT.md for overview
   - Parse TODO.md for completion status
   - Review USERFINDINGS.md for critical issues
   - Check prompt.md for active focus

2. **Evaluate Metrics**
   - Calculate TODO completion percentage
   - Count critical vs. minor issues
   - Assess code stability (compilation status)
   - Review feature completeness

3. **Generate State Report**
   ```
   ## Project State Report
   
   - TODO Progress: X/Y tasks (Z%)
   - Critical Issues: N
   - Feature Complete: P%
   - Last Build: [status]
   ```

### Phase 2: Market Research

1. **Identify Market Segment**
   - Terminal visualizers
   - Audio-reactive applications
   - ASCII art tools

2. **Analyze Competition**
   - List 3-5 similar projects
   - Document features, pricing, audience
   - Identify gaps in market

3. **Identify Opportunities**
   - Unique features ascii-cam offers
   - Underserved user needs
   - Differentiation strategy

4. **Document Findings**
   - Create research/market_analysis.md
   - Update PROJECT.md with market position

### Phase 3: Release Planning

1. **Define Release Criteria**
   - [ ] All critical bugs fixed
   - [ ] Core features implemented
   - [ ] Cross-platform builds working
   - [ ] Documentation complete
   - [ ] Performance acceptable (<100ms latency)

2. **Create Roadmap**
   - Milestone 1: Bug fixes (Week 1-2)
   - Milestone 2: Core features (Week 3-4)
   - Milestone 3: Polish (Week 5-6)
   - Milestone 4: Release (Week 7-8)

3. **Identify Blockers**
   - Technical blockers
   - Resource constraints
   - External dependencies

4. **Create Action Plan**
   - Prioritize tasks
   - Assign to appropriate agents
   - Set milestones in TODO.md

---

## Agent Connections

### → Debug (BugHunter)
**Input:** Critical issues needing investigation
**Output:** Root cause analysis, fix recommendations

```
When to invoke:
- USERFINDINGS contains critical bugs
- Runtime crashes detected
- Unclear error sources
```

### → Research
**Input:** Technical questions, library comparisons
**Output:** Research documents, implementation options

```
When to invoke:
- Need to evaluate technical approaches
- Library selection needed
- Algorithm research required
```

### → Management (AgentManager)
**Input:** Prioritized task list, release plan
**Output:** Task coordination, progress tracking

```
When to invoke:
- Ready to execute release plan
- Need to queue tasks for agents
- Progress monitoring required
```

---

## Output Formats

### Project State Report
```markdown
# Project State: ascii-cam

## Metrics
- **TODO Completion**: 3/10 (30%)
- **Critical Issues**: 1 (MIDI crash)
- **Feature Complete**: ~60%
- **Last Build**: ✅ Compiles

## Strengths
- Audio architecture solid
- Cross-platform foundation
- Character set variety

## Weaknesses
- BPM counter unstable
- MIDI crashes
- Missing UI features

## Next Priority
1. Fix MIDI crash (Critical)
2. Add quit confirmation
3. Stabilize BPM
```

### Market Analysis
```markdown
# Market Analysis: ASCII Audio Visualizers

## Competitors
1. **Project A** - Features, pricing, audience
2. **Project B** - Features, pricing, audience
...

## Opportunities
- Real-time audio reactivity
- Multiple character sets
- MIDI/OSC support (planned)

## Strategy
- Position as feature-rich terminal tool
- Emphasize cross-platform
- Target: musicians, VJs, hobbyists
```

### Release Plan
```markdown
# Release Plan: v1.0

## Release Criteria
- [ ] 0 critical bugs
- [ ] All core features working
- [ ] Linux/macOS builds
- [ ] Basic documentation
- [ ] < 100ms audio latency

## Milestones
### M1: Stability (Week 1-2)
- [ ] Fix MIDI crash
- [ ] Fix BPM instability
- [ ] Add quit confirmation

### M2: Features (Week 3-4)
- [ ] Separate control window
- [ ] MIDI interface
- [ ] OSC protocol

### M3: Polish (Week 5-6)
- [ ] Windows build
- [ ] Performance optimization
- [ ] Documentation

### M4: Release (Week 7-8)
- [ ] Final testing
- [ ] Release build
- [ ] Announcement
```

---

## Progress Reporting

After state analysis:
- "Project state analyzed: X% complete, N critical issues"
- Summary of strengths/weaknesses

After market research:
- "Market analysis complete: N competitors, X opportunities"
- Market position recommendation

After release planning:
- "Release plan created: M milestones, N tasks"
- Timeline estimate

After agent coordination:
- "Tasks queued for execution"
- Next milestone details

---

## Key Files

| File | Purpose |
|------|---------|
| `PROJECT.md` | Project overview |
| `TODO.md` | Task tracking |
| `USERFINDINGS.md` | User issues |
| `research/` | Research documents |
| `.management/bug_index.md` | Bug tracking |

---

## Integration Points

| Agent | Direction | Purpose |
|-------|-----------|---------|
| **BugHunter** | Input → Output | Issue investigation results |
| **Research** | Input → Output | Technical research findings |
| **AgentManager** | Output → Input | Task execution coordination |

---

## Guidelines

1. **Stay within workspace** - Only access project files
2. **Use data-driven decisions** - Base analysis on metrics
3. **Connect to agents** - Route work to appropriate agents
4. **Document everything** - Write findings to research/
5. **Be actionable** - Plans must be executable
6. **Update TODO.md** - Ensure task list reflects priorities

---

## ascii-cam Specific Context

**Current State (as of 2026-04-12):**
- TODO: 0/10 complete (0%)
- Critical: MIDI menu crash
- Medium: Missing g key in submenus, BPM instability
- Features: Audio pipeline working, camera input partial
- Platforms: Linux OK, macOS OK, Windows TODO

**Target:** Release-ready v1.0 with:
- Stable core features
- Cross-platform builds
- Basic documentation

---

*Last Updated: 2026-04-12*