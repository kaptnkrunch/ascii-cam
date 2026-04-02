# Caretaker Skill

**Purpose:** Gather, clean, and integrate workflow documentation from projects into the template system with version control, non-destructive merging, and backups.

---

## Workflow Phases

### Phase 1: Gather
1. Accept project path as input
2. Scan for MD files: `*.md`, `AGENTS.md`, `TODO.md`, `PROJECT.md`, `skill*.md`, `.management/*.md`, `.breedingpool/*.md`
3. Copy to staging: `~/.temp/caretaker_staging/`
4. Log gathered files count

### Phase 2: Clean Project Specifics
Clean patterns (regex replacement):
- `[Pp]roject.*Name.*:` → remove lines with specific project names
- `github\.com/[\w-]+` → genericize URLs
- `local.*path` → remove local paths
- `2026-\d{2}-\d{2}` → normalize dates
- Version numbers: `\d+\.\d+(\.\d+)?` → `[VERSION]`
- UUIDs: `[0-9a-f]{8}-[0-9a-f]{4}` → `[UUID]`
- IP addresses: `\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}` → `[IP]`
- File paths: `/home/[\w]+/` → `~/`
- Workspace paths: `/workspace/[\w]+/` → `[WORKSPACE]/`

### Phase 3: Double Check
1. Scan for remaining PII: emails, names, specific IPs
2. Verify no binary/sensitive content
3. Check for template placeholders already present
4. Validate MD syntax (headers, links)

### Phase 4: Version Control & Backup
1. Check if file exists in `~/template/`
2. If exists: create backup `~/template/.backups/{filename}.{timestamp}.bak`
3. Initialize git repo if not exists: `~/template/.git/`
4. Stage changes: `git add {filename}`
5. Commit: `git commit -m "Caretaker: integrate {filename} from {project}"`

### Phase 5: Non-Destructive Merge
1. For conflicts: keep both versions with markers
```
<<<<<<< INCOMING
{new content}
=======
{existing content}
>>>>>>> EXISTING
```
2. Create merge report: `~/.temp/caretaker_merge_{timestamp}.log`
3. Update VERSION markers incrementally

---

## Configuration

```yaml
staging_dir: ~/.temp/caretaker_staging
backup_dir: ~/template/.backups
template_dir: ~/template
clean_patterns:
  - project_name
  - github_urls
  - local_paths
  - version_numbers
  - uuids
merge_strategy: keep_both  # or overwrite, keep_existing
```

---

## Usage

```bash
# Run caretaker on a project
caretaker /path/to/project

# Dry run (just gather and clean, no merge)
caretaker /path/to/project --dry-run

# Force overwrite existing files
caretaker /path/to/project --force

# View merge report
cat ~/.temp/caretaker_merge_*.log

# Restore from backup
caretaker --restore ~/template/.backups/TODO.md.20260402.bak
```

---

## File Categories

| Category | Files | Merge Strategy |
|----------|-------|---------------|
| Core | AGENTS.md, PROJECT.md, TODO.md | keep_both |
| Skills | skill*.md, .breedingpool/*.md | keep_both |
| Research | research/*.md | overwrite |
| Workflows | .management/*.md | keep_both |
| Config | CODE.md, prompt.md | keep_existing |

---

## Safety Features

1. **Backup First**: Always backup before overwrite
2. **Git History**: Full version control on template
3. **Merge Reports**: Detailed logs of all changes
4. **Dry Run**: Test without committing
5. **Restore**: Rollback capability

---

*Last Updated: 2026-04-02*
