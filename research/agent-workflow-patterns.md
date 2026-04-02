# Research: Agent Workflow Patterns

## Overview

Research into best practices for AI agent workflows, skill file design, and orchestration patterns for improving agent capabilities.

---

## Key Findings

### 1. Skill Composition & Multi-Skill Workflows
- **Pattern**: Combine multiple specialized skills for complex tasks
- **Approach**: Sequential skill execution with shared context
- **Use Case**: Research -> Plan -> Implement -> Test pipeline

### 2. Pipeline Pattern
- **Pattern**: Sequential agent stages for data processing
- **Approach**: Each stage transforms output for next stage
- **Use Case**: Research Agent -> Engineer -> Debug Agent -> Manager

### 3. Workflow Orchestrator
- **Pattern**: Dedicated orchestrator skill to coordinate other agents
- **Approach**: Task decomposition, agent assignment, result aggregation
- **Use Case**: Complex multi-step implementations

### 4. Agentic Architecture Patterns
- **Structure**:
  - **Skills**: Markdown workflows for specific tasks
  - **Agents**: Configured with skill access and path boundaries
  - **Commands**: Direct invocation triggers
  - **Hooks**: Event-driven automation

---

## Best Practices from Research

### Skill File Structure
- Clear responsibility definition
- Phase-based workflow (Research -> Plan -> Execute -> Verify)
- Progress reporting templates
- Integration points documented

### Path Configuration
- Explicit read/write boundaries per agent
- Exclusion patterns for build artifacts
- Max depth limits to prevent overreach

### Self-Refinement
- Periodic workflow improvement cycles
- Pattern research triggers
- Findings integration process
- Documentation updates

---

## Recommendation

The [AGENT_NAME] skill should include:
1. **Self-Improvement Phase**: Dedicated phase for workflow refinement
2. **Research Triggers**: Clear conditions for pattern research
3. **Findings Integration**: Process for updating skill files
4. **Orchestration Support**: Ability to coordinate other agents

### Integration Points
- Add to [AGENT_CONFIG]: [Agent] can manage other agents
- Add to [TASK_FILE]: Self-improvement as ongoing task
- Add to .management/: New orchestration patterns

---

## Version

- **Date**: [date]
- **Source**: [Research sources]
- **Status**: [Integration status]
