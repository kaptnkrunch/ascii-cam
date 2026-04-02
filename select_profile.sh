#!/bin/bash
# Profile Selection Routine
# Usage: ./select_profile.sh [TASK_TYPE] [PROJECT_TYPE]
# 
# Examples:
#   ./select_profile.sh bug_fix hardware
#   ./select_profile.sh research renderer
#   ./select_profile.sh implement game_engine

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROFILES_DIR="$SCRIPT_DIR/../profiles_en"

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

echo_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
echo_primary() { echo -e "${CYAN}[PRIMARY]${NC} $1"; }
echo_secondary() { echo -e "${YELLOW}[SECONDARY]${NC} $1"; }
echo_principle() { echo -e "${GREEN}[PHILOSOPHY]${NC} $1"; }

# Task type detection and profile mapping
case "$1" in
    bug_fix|debug|crash)
        echo_primary "Ada Lovelace (Algorithm debugging)"
        echo_principle "Systems thinking - trace through entire system"
        echo ""
        echo_secondary "Steve Wozniak (Elegant fix)"
        echo_principle "Efficiency as aesthetics - simplest solution"
        echo ""
        echo_secondary "Terry Davis (Bare-metal debugging)"
        echo_principle "Trace to lowest level, own entire stack"
        ;;
        
    research|analysis|investigate)
        echo_primary "Alan Turing (First principles)"
        echo_principle "Reduce to simplest form, prove limits"
        echo ""
        echo_secondary "Ada Lovelace (Systems thinking)"
        echo_principle "Look for analogies between concepts"
        echo ""
        echo_secondary "Tim Berners-Lee (Protocol design)"
        echo_principle "Simple protocols everyone can use"
        ;;
        
    implement|build|create)
        echo_primary "John Carmack (Shipping)"
        echo_principle "Benchmark-driven, measure everything"
        echo ""
        echo_secondary "Steve Wozniak (Efficiency)"
        echo_principle "Reduce to essential, elegant over pragmatic"
        echo ""
        echo_secondary "Andy Gavin (Infrastructure first)"
        echo_principle "Build tools before product"
        ;;
        
    performance|optimize|tuning)
        echo_primary "Andy Gavin (Max control)"
        echo_principle "Don't trust frameworks, build own tools"
        echo ""
        echo_secondary "John Carmack (Benchmark)"
        echo_principle "First-principles physics, numbers driven"
        echo ""
        echo_secondary "Terry Davis (Bare-metal)"
        echo_principle "Eliminate abstraction layers"
        ;;
        
    design|ux|creative)
        echo_primary "Tom Hall (Fun-first)"
        echo_principle "Test with non-gamers, humour as structure"
        echo ""
        echo_secondary "John Romero (Gamefeel)"
        echo_principle "Play continuously while building"
        echo ""
        echo_secondary "Hideo Kojima (Lateral thinking)"
        echo_principle "Turn constraints into features"
        ;;
        
    platform|infrastructure|ecosystem)
        echo_primary "Gabe Newell (Economic thinking)"
        echo_principle "Community as resource, incentives over rules"
        echo ""
        echo_secondary "Tim Berners-Lee (Decentralisation)"
        echo_principle "No central control, universal access"
        echo ""
        echo_secondary "Steve Wozniak (Hacker ethos)"
        echo_principle "Technology for everyone"
        ;;
        
    network|protocol|api)
        echo_primary "Tim Berners-Lee (Simplicity)"
        echo_principle "Simple protocol, rough consensus and running code"
        echo ""
        echo_secondary "Alan Turing (Abstraction)"
        echo_principle "Universal machines, prove what's possible"
        ;;
        
    compiler|os|lowlevel)
        echo_primary "Terry Davis (Radical independence)"
        echo_principle "Everything from scratch, own language, own kernel"
        echo ""
        echo_secondary "Ada Lovelace (Separation)"
        echo_principle "Define operations independent of machine"
        echo ""
        echo_secondary "Andy Gavin (Tool building)"
        echo_principle "Build the tools before the product"
        ;;
        
    game|interactive|narrative)
        echo_primary "Hideo Kojima (Transmedia)"
        echo_principle "Combine all media forms, cast as narrative"
        echo ""
        echo_secondary "Tom Hall (World building)"
        echo_principle "Internal consistency, characters have backstories"
        echo ""
        echo_secondary "John Romero (Spatial design)"
        echo_principle "Architecture as emotional experience"
        ;;
        
    business|operations|strategy)
        echo_primary "Satya Nadella (Growth mindset)"
        echo_principle "Customer-back, learn-it-all"
        echo ""
        echo_secondary "Steve Jobs (Vision)"
        echo_principle "Simplicity is the ultimate sophistication"
        echo ""
        echo_secondary "Warren Buffett (Value)"
        echo_principle "Price is what you pay, value is what you get"
        ;;
        
    sales|marketing|customers)
        echo_primary "Marc Benioff (Trust-based)"
        echo_principle "The only way to be successful is to help others"
        echo ""
        echo_secondary "Steve Jobs (Presentation)"
        echo_principle "Design is how it works"
        ;;
        
    finance|investing|budget)
        echo_primary "Warren Buffett (Long-term)"
        echo_principle "The stock market transfers money from impatient to patient"
        echo ""
        echo_secondary "Satya Nadella (Growth)"
        echo_principle "Fall in love with the problem"
        ;;
        
    hr|hiring|culture)
        echo_primary "Patty McCord (Freedom)"
        echo_principle "Hire smart people and get out of their way"
        echo ""
        echo_secondary "Steve Jobs (A-players)"
        echo_principle "Stay hungry, stay foolish"
        ;;
        
    legal|compliance|ip)
        echo_primary "Lawrence Lessig (Code is law)"
        echo_principle "The problem is not over-regulation, under-regulation"
        echo ""
        echo_secondary "Bruce Schneier (Security)"
        echo_principle "Security theater doesn't work"
        ;;
        
    security|privacy|trust)
        echo_primary "Bruce Schneier (Security-first)"
        echo_principle "Complexity is the enemy of security"
        echo ""
        echo_secondary "Lawrence Lessig (Trust)"
        echo_principle "Code is law"
        ;;
        
    data|analytics|metrics)
        echo_primary "Hilary Mason (Decision-driven)"
        echo_principle "The best model is the one that gets used"
        echo ""
        echo_secondary "Warren Buffett (Cash flow)"
        echo_principle "Focus on owner earnings"
        ;;
        
    mobile|app|consumer)
        echo_primary "Susan Wojcicki (Creator-first)"
        echo_principle "Invest in creators, they are the future"
        echo ""
        echo_secondary "Steve Jobs (Perfectionism)"
        echo_principle "Simplicity is the ultimate sophistication"
        ;;
        
    qa|testing|quality)
        echo_primary "James Bach (Exploratory)"
        echo_principle "Testing is an intellectual activity"
        echo ""
        echo_secondary "Cem Kaner (Context-driven)"
        echo_principle "A test case is a question you ask the software"
        ;;
        
    help|--help|-h)
        echo "Profile Selection Routine"
        echo ""
        echo "Usage: $0 [TASK_TYPE] [PROJECT_TYPE]"
        echo ""
        echo "Task Types (Technical):"
        echo "  bug_fix/debug/crash     - Debugging and fixing issues"
        echo "  research/analysis      - Research and investigation"
        echo "  implement/build/create - Feature implementation"
        echo "  performance/optimize   - Performance tuning"
        echo "  network/protocol       - Network and API design"
        echo "  compiler/os/lowlevel   - Systems programming"
        echo ""
        echo "Task Types (Design/Creative):"
        echo "  design/ux/creative     - User experience and design"
        echo "  game/narrative        - Game design and interactive"
        echo ""
        echo "Task Types (Platform):"
        echo "  platform/infra        - Platform and infrastructure"
        echo "  mobile/app            - Mobile development"
        echo ""
        echo "Task Types (Business):"
        echo "  business/operations    - Strategy and operations"
        echo "  sales/marketing        - Sales and customer relations"
        echo "  finance/investing      - Finance and budgets"
        echo "  hr/hiring              - HR and culture"
        echo "  legal/compliance       - Legal and IP"
        echo "  security/privacy      - Security and trust"
        echo "  data/analytics        - Data and metrics"
        echo "  qa/testing            - Quality assurance"
        echo ""
        echo "Examples:"
        echo "  $0 bug_fix hardware"
        echo "  $0 research ai_ml"
        echo "  $0 business operations"
        echo "  $0 security privacy"
        echo ""
        echo "Output: Primary persona + philosophy, secondary options"
        ;;
        
    *)
        echo_info "Analyzing task: $1"
        echo ""
        echo_primary "Steve Jobs (General purpose - vision & execution)"
        echo_principle "Simplicity is the ultimate sophistication"
        echo ""
        echo_secondary "Satya Nadella (Operations)"
        echo_principle "The most important thing is to fall in love with the problem"
esac

echo ""
echo_info "To read full profile: cat $PROFILES_DIR/01_Ada_Lovelace.md"
echo "Corresponding persona: Matching profile based on task type"