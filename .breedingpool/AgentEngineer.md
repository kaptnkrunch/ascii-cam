# AgentEngineer Skill

*For creating and authoring new agent skills to support project execution*

---

## Overview

The AgentEngineer skill designs, creates, and validates new agent skills needed for the [PROJECT_NAME] project. It analyzes project requirements, identifies missing capabilities, and produces complete skill files with proper workflows, responsibilities, and integration points.

---

## Core Responsibilities

### 1. Skill Gap Analysis
- Review [PROJECT_TRACKING_FILE] and [TASK_FILE] for unassigned capabilities
- Identify skills needed for pending tasks
- Determine skill dependencies and relationships

### 2. Skill Authoring
- Create complete skill files with workflow phases
- Define responsibilities, key files, and integration points
- Include research formats, implementation plan templates
- Add progress reporting and guidelines

### 3. Skill Validation
- Verify skill file structure matches project standards
- Check cross-references to existing files
- Ensure no conflicts with existing agents
- Validate path configurations

### 4. Integration
- Update [AGENT_CONFIG] with new agent configuration
- Add path boundaries and exclusions
- Update .management/ workflow files if needed
- Register skill in project documentation

---

## Workflow

### Phase 1: Analysis

1. **Identify Need**
   - Read [PROJECT_TRACKING_FILE] for pending tasks
   - Check [TASK_FILE] for unassigned work
   - Review existing skills in .management/

2. **Define Requirements**
   - What capabilities are missing?
   - What files does the skill need access to?
   - What other skills will it work with?
   - What is the skill's output format?

3. **Research Patterns**
   - Check existing skill files for conventions
   - Review agent workflow patterns in research/
   - Identify best practices for skill authoring

### Phase 2: Authoring

1. **Create Skill File**
   - Use standard skill template structure
   - Define clear responsibility sections
   - Add phase-based workflow
   - Include progress reporting format

2. **Define Structure**
   ```markdown
   # [Skill Name]
   
   *Tagline describing purpose*
   
   ## Overview
   ## Core Responsibilities
   ## Workflow
   ## Key Files
   ## [Format Templates]
   ## Progress Reporting
   ## Integration
   ## Guidelines
   ```

3. **Add Integration Points**
   - List files the skill reads/writes
   - Define agent configuration for [AGENT_CONFIG]
   - Specify path boundaries

### Phase 3: Validation

1. **Check Structure**
   - All required sections present
   - Consistent formatting with existing skills
   - No broken references

2. **Verify Paths**
   - All file references exist or are planned
   - Path boundaries don't conflict with other agents
   - Exclusions are appropriate

3. **Test Integration**
   - Skill can be referenced by other agents
   - [AGENT_CONFIG] entry is correct
   - Workflow files updated if needed

### Phase 4: Deployment

1. **Write Skill File**
   - Save to .breedingpool/ (new skills)
   - Move to .management/ (approved skills)

2. **Update Configuration**
   - Add agent entry to [AGENT_CONFIG]
   - Update path exclusions table
   - Add to workspace boundaries

3. **Document**
   - Add to [PROJECT_TRACKING_FILE] research/skills section
   - Update [TASK_FILE] with new skill tasks
   - Note in Recent Work

---

## Skill Template

```markdown
# [Skill Name]

*For [brief purpose description]*

---

## Overview

[2-3 sentence description of what this skill does and why it exists]

---

## Core Responsibilities

### 1. [Primary Responsibility]
- [Specific capability]
- [Specific capability]

### 2. [Secondary Responsibility]
- [Specific capability]
- [Specific capability]

---

## Workflow

### Phase 1: [Phase Name]

1. **Step Name**
   - Detail
   - Detail

### Phase 2: [Phase Name]

1. **Step Name**
   - Detail
   - Detail

---

## Key Files

| File | Purpose |
|------|---------|
| `path/to/file` | Description |

---

## [Format/Template Section]

[Include any templates this skill produces or consumes]

---

## Progress Reporting

After [phase]:
- "Message format"
- Details

---

## Integration

This skill works with:
- [Other skill/agent]
- [File references]

---

## Guidelines

1. [Rule or best practice]
2. [Rule or best practice]

---

*Last Updated: [date]*
```

---

## Output Format

When creating a new skill, produce:

1. **Skill File** - Complete markdown skill definition
2. **[AGENT_CONFIG] Entry** - Agent configuration block
3. **Integration Notes** - How the skill connects to existing workflow

---

## Key Files

| File | Purpose |
|------|---------|
| `.breedingpool/` | New/prototype skills |
| `.management/` | Approved active skills |
| `[AGENT_CONFIG]` | Agent configuration |
| `[PROJECT_TRACKING_FILE]` | Task tracking |
| `[TASK_FILE]` | Detailed tasks |

---

## Progress Reporting

After analysis:
- "Skill gap identified: [capability needed]"
- Rationale and dependencies

After authoring:
- "Skill created: [name]"
- Sections included and file count

After validation:
- "Skill validated: [name]"
- Checks passed

After deployment:
- "Skill deployed: [name]"
- Files updated and agent configured

---

## Integration

The AgentEngineer works with:
- [PROJECT_TRACKING_FILE] for task requirements
- [TASK_FILE] for detailed capability need
- Existing .management/ skills for patterns
- [AGENT_CONFIG] for agent configuration
- research/[WORKFLOW_PATTERNS_FILE] for best practices

---

## Guidelines

1. Always follow existing skill file structure
2. Include all standard sections (Overview, Responsibilities, Workflow, etc.)
3. Define clear path boundaries for new agents
4. Validate against existing skills for conflicts
5. Keep skills focused - split large skills into smaller ones
6. Document integration points explicitly
7. Use consistent formatting across all skills

---

*Last Updated: [date]*
