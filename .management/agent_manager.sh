#!/bin/bash
# AgentManager Routine Script
# Usage: ./agent_manager.sh [TASK_TYPE] [TASK_DESCRIPTION]
# 
# Examples:
#   ./agent_manager.sh bug "Fix MIDI menu crash"
#   ./agent_manager.sh research "Audio latency optimization"
#   ./agent_manager.sh build "Add BPM counter"
#   ./agent_manager.sh test "Verify audio input"
#   ./agent_manager.sh doc "Update README"

set -e

PROJECT_ROOT="/home/crunch/ascii-cam"
cd "$PROJECT_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
echo_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
echo_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
echo_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Function to load context files
load_context() {
    echo_info "Loading context files..."
    
    echo "=== PROJECT.md ===" > /tmp/agent_context.txt
    cat PROJECT.md >> /tmp/agent_context.txt 2>/dev/null || echo "(not found)" >> /tmp/agent_context.txt
    
    echo "" >> /tmp/agent_context.txt
    echo "=== TODO.md ===" >> /tmp/agent_context.txt
    cat TODO.md >> /tmp/agent_context.txt 2>/dev/null || echo "(not found)" >> /tmp/agent_context.txt
    
    echo "" >> /tmp/agent_context.txt
    echo "=== USERFINDINGS.md ===" >> /tmp/agent_context.txt
    cat USERFINDINGS.md >> /tmp/agent_context.txt 2>/dev/null || echo "(not found)" >> /tmp/agent_context.txt
    
    echo "Context loaded to /tmp/agent_context.txt"
}

# Function to run BugHunter
run_bughunter() {
    local task="$1"
    echo_info "Invoking BugHunter for: $task"
    
    load_context
    
    cat > /tmp/bughunter_task.txt << EOF
TASK: $task

CONTEXT:
$(cat /tmp/agent_context.txt)

INSTRUCTIONS:
1. Find the exact code location of the bug
2. Document in .management/bug_index.md
3. Implement fix
4. Verify with cargo check --release
5. Update TODO.md status if applicable
EOF
    
    echo_success "BugHunter task prepared"
    echo "Task file: /tmp/bughunter_task.txt"
    echo "Bug index: .management/bug_index.md"
}

# Function to run Researcher
run_researcher() {
    local task="$1"
    echo_info "Invoking Researcher for: $task"
    
    load_context
    
    cat > /tmp/researcher_task.txt << EOF
TASK: $task

CONTEXT:
$(cat /tmp/agent_context.txt)

INSTRUCTIONS:
1. Research the topic thoroughly
2. Document findings in research/ directory
3. Create implementation plan
4. Update PROJECT.md with research findings
5. Add any new tasks to TODO.md
EOF
    
    echo_success "Researcher task prepared"
    echo "Task file: /tmp/researcher_task.txt"
    echo "Output: research/"
}

# Function to run Builder
run_builder() {
    local task="$1"
    echo_info "Invoking Builder for: $task"
    
    load_context
    
    cat > /tmp/builder_task.txt << EOF
TASK: $task

CONTEXT:
$(cat /tmp/agent_context.txt)

INSTRUCTIONS:
1. Implement the feature/bugfix
2. Test with cargo build --release
3. Run cargo check for errors
4. Test functionality
5. Update TODO.md when complete
6. Update prompt.md with status
EOF
    
    echo_success "Builder task prepared"
    echo "Task file: /tmp/builder_task.txt"
}

# Function to run Tester
run_tester() {
    local task="$1"
    echo_info "Invoking Tester for: $task"
    
    load_context
    
    cat > /tmp/tester_task.txt << EOF
TASK: $task

CONTEXT:
$(cat /tmp/agent_context.txt)

INSTRUCTIONS:
1. Verify implementation works
2. Run: cargo check --release
3. Run: cargo build --release
4. Run: cargo run --release (if applicable)
5. Check for runtime errors
6. Report results
EOF
    
    echo_success "Tester task prepared"
    echo "Task file: /tmp/tester_task.txt"
}

# Function to run Documenter
run_documenter() {
    local task="$1"
    echo_info "Invoking Documenter for: $task"
    
    load_context
    
    cat > /tmp/documenter_task.txt << EOF
TASK: $task

CONTEXT:
$(cat /tmp/agent_context.txt)

INSTRUCTIONS:
1. Update relevant documentation
2. Check README.md, CODE.md, TROUBLESHOOTING.md
3. Add comments to source code if needed
4. Update CHANGELOG if applicable
EOF
    
    echo_success "Documenter task prepared"
    echo "Task file: /tmp/documenter_task.txt"
}

# Function to handle compilation errors
handle_compile_error() {
    echo_info "Handling compilation error..."
    
    # Run cargo check to get errors
    if cargo check --release 2>&1 | tee /tmp/compile_errors.txt | grep -q "^error"; then
        echo_error "Compilation errors found!"
        cat /tmp/compile_errors.txt | grep "^error"
        run_bughunter "Fix compilation errors from cargo check"
    else
        echo_success "No compilation errors - build OK!"
        echo_info "Warnings (non-blocking):"
        grep -E "^warning" /tmp/compile_errors.txt | head -5
    fi
}

# Main script logic
case "$1" in
    bug)
        run_bughunter "$2"
        ;;
    research)
        run_researcher "$2"
        ;;
    build)
        run_builder "$2"
        ;;
    test)
        run_tester "$2"
        ;;
    doc)
        run_documenter "$2"
        ;;
    compile)
        handle_compile_error
        ;;
    help|--help|-h)
        echo "AgentManager Routine Script"
        echo ""
        echo "Usage: $0 [COMMAND] [TASK_DESCRIPTION]"
        echo ""
        echo "Commands:"
        echo "  bug [desc]      - Invoke BugHunter to find/fix bugs"
        echo "  research [desc] - Invoke Researcher for technical investigation"
        echo "  build [desc]    - Invoke Builder for implementation"
        echo "  test [desc]    - Invoke Tester for verification"
        echo "  doc [desc]     - Invoke Documenter for docs"
        echo "  compile        - Check for compilation errors"
        echo "  help           - Show this help"
        echo ""
        echo "Examples:"
        echo "  $0 bug 'Fix MIDI menu crash'"
        echo "  $0 research 'Audio FFT optimization'"
        echo "  $0 build 'Add BPM counter'"
        echo "  $0 compile"
        ;;
    *)
        echo_error "Unknown command: $1"
        echo "Run '$0 help' for usage"
        exit 1
        ;;
esac

echo ""
echo_info "Next steps:"
echo "  1. Review task file in /tmp/"
echo "  2. Execute with appropriate agent"
echo "  3. Verify with: cargo check --release"
echo "  4. Update TODO.md status"