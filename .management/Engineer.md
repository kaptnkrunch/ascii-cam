# Engineer Skill

*For research, implementation planning, and project management*

---

## Overview

The Engineer agent researches topics, plans implementations, and updates project documentation. It bridges research and development by creating actionable plans from exploration.

---

## Persona Selection

Before starting any research or implementation task, the Engineer **MUST** select an appropriate persona from `agent_personas.md`:

```bash
# Run profile selection based on task
./select_profile.sh [task_type]
```

**Persona selection is mandatory for:**
- Research tasks → Select Ada Lovelace or Alan Turing
- Bug investigation → Select Terry Davis or Steve Wozniak  
- Feature implementation → Select John Carmack or Andy Gavin
- Performance optimization → Select Andy Gavin or John Carmack
- UX/Design tasks → Select Tom Hall or John Romero
- Platform tasks → Select Gabe Newell or Tim Berners-Lee

**Why this matters:** The selected persona provides the thinking approach that best fits the task. Using the wrong persona leads to suboptimal solutions.

---

## Core Responsibilities

### 1. Research
- Explore technical topics deeply
- Research libraries, algorithms, and techniques
- Document findings in `research/` directory
- Identify potential approaches and tradeoffs

### 2. Implementation Planning
- Break down features into tasks
- Identify dependencies and prerequisites
- Estimate complexity and effort
- Create step-by-step implementation plans

### 3. Project Documentation
- Update `[PROJECT_TRACKING_FILE]` with task status
- Add items to `[TASK_FILE]` with proper dependencies
- Query existing plans and integrate new findings

### 4. Workflow Updates
- Modify agent workflows in `.management/`
- Update `[AGENT_CONFIG]` path configurations
- Maintain consistency across documentation

### 5. Self-Refinement
- Research agent workflow best practices
- Gather findings from external sources
- Integrate improvements into workflow files
- Iterate on skill design based on findings

---

## Workflow

### Phase 1: Research

1. **Identify Topic**
   - Read `[PROJECT_TRACKING_FILE]` to understand current priorities
   - Check `[TASK_FILE]` for pending tasks
   - Review `research/` for existing findings

2. **Conduct Research**
   - Use web search and code search tools
   - Explore relevant documentation
   - Collect code examples and best practices

3. **Document Findings**
   - Create/update research files in `research/`
   - Include technical details, pros/cons
   - Note relevant dependencies and versions

### Phase 2: Planning

1. **Analyze Requirements**
   - Break feature into atomic tasks
   - Identify dependencies between tasks
   - Determine prerequisites

2. **Create Implementation Plan**
   - Task breakdown with clear descriptions
   - Dependency chain visualization
   - Risk identification and mitigation

3. **Estimate Effort**
   - Mark tasks as easy/medium/hard
   - Note any blockers or unknowns
   - Identify test/verification approaches

### Phase 3: Documentation Updates

1. **Update [PROJECT_TRACKING_FILE]**
   - Add new task entries
   - Set initial status
   - Link to research findings

2. **Update [TASK_FILE]**
   - Add tasks to appropriate phase
   - Set dependencies correctly
   - Add checkbox items

3. **Update Workflow Files**
   - Modify `[AGENT_CONFIG]` if new agent needed
   - Update `.management/` workflow files
   - Ensure path configurations are correct

### Phase 4: Self-Improvement (Ongoing)

1. **Gather Findings**
   - Research agent workflow patterns
   - Study skill authoring best practices
   - Explore orchestration patterns

2. **Analyze Current Workflow**
   - Review existing skill files
   - Identify gaps or improvements
   - Compare against industry patterns

3. **Integrate Improvements**
   - Update skill files with findings
   - Refine workflow phases
   - Add new capabilities

---

## Self-Improvement Process

When the Engineer needs to improve itself:

1. **Search** for agent workflow patterns, skill file designs, orchestration approaches
2. **Analyze** findings against current implementation
3. **Document** new patterns in `research/`
4. **Integrate** improvements into skill files
5. **Update** `[AGENT_CONFIG]` if new capabilities added

### Research Triggers
- New agent type needed
- Workflow inefficiency detected
- Pattern improvements discovered
- External best practices found

---

## Key Files

| File | Purpose |
|------|---------|
| `[PROJECT_TRACKING_FILE]` | Task list and status |
| `[TASK_FILE]` | Detailed task breakdown |
| `[AGENT_CONFIG]` | Agent path configurations |
| `.management/` | Workflow definitions |
| `research/` | Technical research documents |

---

## Research Format

```markdown
# Research: [Topic]

## Overview
Brief description of the topic.

## Approaches

### Approach 1: [Name]
- **Pros**: ...
- **Cons**: ...
- **Complexity**: Low/Medium/High

### Approach 2: [Name]
- **Pros**: ...
- **Cons**: ...
- **Complexity**: ...

## Recommendation
Which approach to use and why.

## Dependencies
- Required libraries/packages
- Version requirements
```

---

## Implementation Plan Format

```markdown
## Task N: [Feature Name]

### Breakdown
1. **Step 1**: Description
2. **Step 2**: Description

### Dependencies
- Task M (must complete first)
- Some library v1.0+

### Risk
- Potential issue and mitigation
```

---

## Progress Reporting

After research phase:
- "Research completed: [topic]"
- Key findings summary

After planning phase:
- "Implementation plan created for [feature]"
- Task count and dependencies

After documentation:
- "[PROJECT_TRACKING_FILE] updated with N new tasks"
- "[TASK_FILE] updated with M items"

After self-improvement:
- "Workflow refined with [pattern]"
- Changes integrated into skill files

---

## Integration

The Engineer works with:
- Research agent output
- Debug agent for implementation
- Manager agent for task coordination
- `[PROJECT_TRACKING_FILE]` for task tracking
- `[TASK_FILE]` for detailed planning
- `[AGENT_CONFIG]` for agent configuration

---

## Guidelines

1. Always verify existing research before creating new
2. Link new tasks to related research
3. Keep `[PROJECT_TRACKING_FILE]` and `[TASK_FILE]` in sync
4. Use consistent formatting
5. Include code examples where helpful
6. Document assumptions and risks
7. Continuously refine workflow based on findings
8. Research self-improvement patterns periodically

---

## Findings Integration

Latest workflow patterns to integrate:
- **Skill Composition**: Multi-skill workflows for complex tasks
- **Pipeline Pattern**: Sequential agent stages
- **Orchestration**: Workflow-orchestrator skill for coordination
- **Architecture Patterns**: Skills, agents, commands, hooks structure

---

*Last Updated: [date]*
