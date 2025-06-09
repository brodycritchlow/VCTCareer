#!/bin/bash

# GitHub Actions Local Testing Script for VCTCareer
# Usage: ./scripts/test-github-actions.sh [workflow] [job]

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 VCTCareer GitHub Actions Local Testing${NC}"
echo "=========================================="

# Function to run a specific workflow
run_workflow() {
    local workflow="$1"
    local job="$2"
    local event="${3:-pull_request}"
    
    echo -e "\n${YELLOW}📋 Running workflow: $workflow${NC}"
    if [[ -n "$job" ]]; then
        echo -e "${YELLOW}🎯 Job: $job${NC}"
    fi
    
    local act_cmd="act $event --workflows .github/workflows/$workflow --container-architecture linux/amd64"
    
    if [[ -n "$job" ]]; then
        act_cmd="$act_cmd --job $job"
    fi
    
    echo -e "${BLUE}💻 Command: $act_cmd${NC}"
    echo ""
    
    if ! eval "$act_cmd"; then
        echo -e "\n${RED}❌ Workflow failed!${NC}"
        return 1
    else
        echo -e "\n${GREEN}✅ Workflow completed successfully!${NC}"
        return 0
    fi
}

# Function to validate workflow syntax
validate_workflows() {
    echo -e "\n${YELLOW}🔍 Validating workflow syntax...${NC}"
    
    for workflow in .github/workflows/*.yml; do
        echo -e "${BLUE}Checking: $workflow${NC}"
        if ! act --list --workflows "$workflow" >/dev/null 2>&1; then
            echo -e "${RED}❌ Syntax error in $workflow${NC}"
            return 1
        fi
    done
    
    echo -e "${GREEN}✅ All workflows have valid syntax${NC}"
}

# Function to run backend unit tests only (fastest)
test_backend_unit() {
    echo -e "\n${YELLOW}🦀 Testing Backend Unit Tests (Fast)${NC}"
    run_workflow "pr-ci.yml" "backend-unit-tests"
}

# Function to test frontend only
test_frontend() {
    echo -e "\n${YELLOW}⚛️ Testing Frontend CI${NC}"
    run_workflow "pr-ci.yml" "frontend-ci"
}

# Function to test change detection
test_changes() {
    echo -e "\n${YELLOW}🔍 Testing Change Detection${NC}"
    run_workflow "pr-ci.yml" "changes"
}

# Function to show help
show_help() {
    echo -e "\n${BLUE}Usage:${NC}"
    echo "  $0 [command]"
    echo ""
    echo -e "${BLUE}Commands:${NC}"
    echo "  validate       - Check workflow syntax"
    echo "  backend-unit   - Test backend unit tests (fastest)"
    echo "  frontend       - Test frontend CI"
    echo "  changes        - Test change detection"
    echo "  pr-full        - Run full PR workflow (slow, requires Docker)"
    echo "  main           - Run main branch workflow"
    echo "  list           - List all available jobs"
    echo "  help           - Show this help"
    echo ""
    echo -e "${BLUE}Custom runs:${NC}"
    echo "  $0 workflow job     - Run specific job from workflow"
    echo "  Examples:"
    echo "    $0 pr-ci.yml backend-unit-tests"
    echo "    $0 main-ci.yml backend-check"
}

# Function to list all jobs
list_jobs() {
    echo -e "\n${YELLOW}📋 Available GitHub Actions Jobs:${NC}"
    act --list --workflows .github/workflows/
}

# Main command handling
case "${1:-help}" in
    "validate")
        validate_workflows
        ;;
    "backend-unit"|"backend")
        test_backend_unit
        ;;
    "frontend")
        test_frontend
        ;;
    "changes")
        test_changes
        ;;
    "pr-full")
        echo -e "\n${YELLOW}🔄 Running Full PR Workflow (This will take a while...)${NC}"
        run_workflow "pr-ci.yml" "" "pull_request"
        ;;
    "main")
        echo -e "\n${YELLOW}🏠 Running Main Branch Workflow${NC}"
        run_workflow "main-ci.yml" "" "push"
        ;;
    "list")
        list_jobs
        ;;
    "help"|"--help"|"-h")
        show_help
        ;;
    *.yml)
        # Custom workflow run
        if [[ -n "$2" ]]; then
            run_workflow "$1" "$2"
        else
            run_workflow "$1"
        fi
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $1${NC}"
        show_help
        exit 1
        ;;
esac