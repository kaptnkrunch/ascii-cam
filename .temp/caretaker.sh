#!/bin/bash
# Caretaker - Workflow Documentation Integration Tool
# Version: 1.0.1
# Limited to: /home/crunch/opencode workspace

set -e

WORKSPACE_ROOT="/home/crunch/opencode"
STAGING_DIR="$WORKSPACE_ROOT/.temp/caretaker_staging"
BACKUP_DIR="$WORKSPACE_ROOT/template/.backups"
TEMPLATE_DIR="$WORKSPACE_ROOT/template"
DRY_RUN=false
FORCE=false

usage() {
    cat << EOF
Caretaker - Workflow Documentation Integration Tool
Limited to: $WORKSPACE_ROOT

Usage: caretaker <project_path> [options]

Options:
    --dry-run     Gather and clean only, no merge
    --force       Overwrite existing files without backup
    --restore     Restore from backup file
    -h, --help   Show this help

Examples:
    caretaker $WORKSPACE_ROOT/myproject
    caretaker $WORKSPACE_ROOT/myproject --dry-run
    caretaker --restore $WORKSPACE_ROOT/template/.backups/TODO.md.20260402.bak
EOF
    exit 1
}

log() {
    echo "[Caretaker] $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

init_dirs() {
    mkdir -p "$STAGING_DIR"
    mkdir -p "$BACKUP_DIR"
    mkdir -p "$TEMPLATE_DIR/.git"
    
    # Initialize git if needed
    if [ ! -d "$TEMPLATE_DIR/.git" ]; then
        git init "$TEMPLATE_DIR" 2>/dev/null || true
    fi
}

gather() {
    local project_path="$1"
    local count=0
    
    log "Gathering MD files from: $project_path"
    
    # Clear staging
    rm -rf "$STAGING_DIR"/*
    
    # Find and copy MD files
    while IFS= read -r -d '' file; do
        rel_path="${file#$project_path/}"
        dest="$STAGING_DIR/$rel_path"
        
        mkdir -p "$(dirname "$dest")"
        cp "$file" "$dest"
        
        ((count++))
        log "  + $rel_path"
    done < <(find "$project_path" -type f \( -name "*.md" -o -name "AGENTS.md" -o -name "PROJECT.md" -o -name "TODO.md" \) -print0 2>/dev/null)
    
    log "Gathered $count files"
    echo $count
}

clean_file() {
    local file="$1"
    local timestamp=$(date +%s)
    
    # Create temp file for cleaning
    local temp_file="${file}.clean"
    cp "$file" "$temp_file"
    
    # Clean patterns
    sed -i 's/[Pp]roject[ -][Nn]ame.*/\[PROJECT_NAME\]/g' "$temp_file"
    sed -i 's|https\?://github\.com/[^/]+/[^/]+|https://github.com/[USER]/[REPO]|g' "$temp_file"
    sed -i 's|/home/[^/]+/|~/|g' "$temp_file"
    sed -i 's|/workspace/[^/]+/|~/workspace/|g' "$temp_file"
    sed -i 's|\d{4}-\d{2}-\d{2}|{DATE}|g' "$temp_file"
    sed -i 's|\d+\.\d+(\.\d+)?|[VERSION]|g' "$temp_file"
    sed -i 's|[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}|[UUID]|g' "$temp_file"
    sed -i 's|\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}|[IP]|g' "$temp_file"
    
    # Replace original with cleaned
    mv "$temp_file" "$file"
}

clean_all() {
    local count=0
    
    log "Cleaning project specifics..."
    
    while IFS= read -r -d '' file; do
        clean_file "$file"
        ((count++))
    done < <(find "$STAGING_DIR" -type f -name "*.md" -print0)
    
    log "Cleaned $count files"
    echo $count
}

verify() {
    local issues=0
    
    log "Verifying cleaned files..."
    
    # Check for remaining PII
    while IFS= read -r -d '' file; do
        if grep -qiE 'email|@.*\.' "$file" 2>/dev/null; then
            log "  WARNING: Potential email in $file"
            ((issues++))
        fi
    done < <(find "$STAGING_DIR" -type f -name "*.md" -print0)
    
    # Check for binary content
    while IFS= read -r -d '' file; do
        if file "$file" | grep -q "binary"; then
            log "  ERROR: Binary file found: $file"
            ((issues++))
        fi
    done < <(find "$STAGING_DIR" -type f -print0)
    
    if [ $issues -eq 0 ]; then
        log "Verification passed - 0 issues"
    else
        log "Verification found $issues issues"
    fi
    
    echo $issues
}

backup_existing() {
    local filename=$(basename "$1")
    local backup_path="$BACKUP_DIR/${filename}.$(date +%Y%m%d_%H%M%S).bak"
    
    if [ -f "$TEMPLATE_DIR/$filename" ] && [ "$FORCE" = false ]; then
        cp "$TEMPLATE_DIR/$filename" "$backup_path"
        log "Backed up: $filename -> $backup_path"
        echo "$backup_path"
    fi
}

merge_file() {
    local src="$1"
    local filename=$(basename "$src")
    local dest="$TEMPLATE_DIR/$filename"
    
    log "Merging: $filename"
    
    # Backup existing
    backup_existing "$dest"
    
    if [ -f "$dest" ] && [ "$FORCE" = false ]; then
        # Keep both versions
        cat > "${dest}.incoming" << EOF
<<<<<<< INCOMING ($(date))
$(cat "$src")
=======
$(cat "$dest")
>>>>>>> EXISTING
EOF
        log "  Created merge conflict: ${dest}.incoming"
    else
        # Direct merge
        cp "$src" "$dest"
        log "  Merged: $filename"
    fi
}

git_commit() {
    local filename=$(basename "$1")
    
    cd "$TEMPLATE_DIR"
    git add "$filename" 2>/dev/null || true
    
    if git diff --cached --quiet 2>/dev/null; then
        log "No changes to commit for $filename"
        return
    fi
    
    git commit -m "Caretaker: integrate $filename - $(date)" 2>/dev/null || true
    log "Committed: $filename"
}

create_merge_report() {
    local report="/home/crunch/opencode/.temp/caretaker_merge_$(date +%Y%m%d_%H%M%S).log"
    
    cat > "$report" << EOF
Caretaker Merge Report
======================
Timestamp: $(date)
Project: ${PROJECT_PATH:-unknown}
Files Processed: $(find "$STAGING_DIR" -type f | wc -l)
Backups Created: $(ls -1 "$BACKUP_DIR" 2>/dev/null | wc -l)

Files:
$(find "$STAGING_DIR" -type f -name "*.md" -exec basename {} \; | sort)

EOF
    log "Created merge report: $report"
}

restore_backup() {
    local backup_file="$1"
    
    if [ ! -f "$backup_file" ]; then
        log "ERROR: Backup file not found: $backup_file"
        exit 1
    fi
    
    local filename=$(basename "$backup_file" | sed 's/\.[0-9_]*\.bak$//')
    
    log "Restoring: $filename from $backup_file"
    cp "$backup_file" "$TEMPLATE_DIR/$filename"
    log "Restored: $filename"
}

main() {
    PROJECT_PATH=""
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --force)
                FORCE=true
                shift
                ;;
            --restore)
                restore_backup "$2"
                exit 0
                ;;
            -h|--help)
                usage
                ;;
            -*)
                usage
                ;;
            *)
                PROJECT_PATH="$1"
                shift
                ;;
        esac
    done
    
    if [ -z "$PROJECT_PATH" ]; then
        usage
    fi
    
    # Validate path is within WORKSPACE_ROOT
    if [[ "$PROJECT_PATH" != "$WORKSPACE_ROOT"* ]]; then
        log "ERROR: Project must be within $WORKSPACE_ROOT"
        exit 1
    fi
    
    if [ ! -d "$PROJECT_PATH" ]; then
        log "ERROR: Project path not found: $PROJECT_PATH"
        exit 1
    fi
    
    # Initialize
    init_dirs
    
    # Phase 1: Gather
    gather_count=$(gather "$PROJECT_PATH")
    log "Phase 1 complete: gathered $gather_count files"
    
    # Phase 2: Clean
    clean_count=$(clean_all)
    log "Phase 2 complete: cleaned $clean_count files"
    
    # Phase 3: Verify
    issues=$(verify)
    log "Phase 3 complete: $issues issues found"
    
    if [ "$DRY_RUN" = true ]; then
        log "DRY RUN - skipping merge"
        exit 0
    fi
    
    # Phase 4: Merge
    while IFS= read -r -d '' file; do
        merge_file "$file"
        git_commit "$file"
    done < <(find "$STAGING_DIR" -type f -name "*.md" -print0)
    
    # Phase 5: Report
    create_merge_report
    
    log "Caretaker complete!"
}

main "$@"
