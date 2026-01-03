#!/bin/bash
# Master test runner for all rninja tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║           rninja Complete Test Suite                     ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Build rninja first
echo -e "${BLUE}Building rninja...${NC}"
cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml" 2>&1 | tail -3
echo ""

TOTAL_PASSED=0
TOTAL_FAILED=0
TOTAL_SKIPPED=0

run_suite() {
    local name="$1"
    local script="$2"

    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}  $name${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    if [[ -x "$script" ]]; then
        if bash "$script"; then
            ((TOTAL_PASSED++))
        else
            ((TOTAL_FAILED++))
        fi
    else
        echo -e "${YELLOW}SKIP: $script not found${NC}"
        ((TOTAL_SKIPPED++))
    fi
    echo ""
}

# Make all scripts executable
chmod +x "$PROJECT_ROOT/scripts/"*.sh 2>/dev/null || true
chmod +x "$SCRIPT_DIR/generators/"*.sh 2>/dev/null || true
chmod +x "$SCRIPT_DIR/integration/"*.sh 2>/dev/null || true
chmod +x "$PROJECT_ROOT/benchmarks/"*.sh 2>/dev/null || true

# Run test suites
run_suite "Compatibility Tests (Basic)" "$PROJECT_ROOT/scripts/compat_test.sh"
run_suite "Compatibility Tests (Fuzzy)" "$PROJECT_ROOT/scripts/fuzzy_compat_test.sh"
run_suite "Generator Tests" "$SCRIPT_DIR/generators/run_all.sh"
run_suite "Integration Tests" "$SCRIPT_DIR/integration/run_all.sh"

echo -e "${CYAN}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                    Final Summary                         ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Test Suites ${GREEN}Passed${NC}:  $TOTAL_PASSED"
echo -e "  Test Suites ${RED}Failed${NC}:  $TOTAL_FAILED"
echo -e "  Test Suites ${YELLOW}Skipped${NC}: $TOTAL_SKIPPED"
echo ""

if [[ $TOTAL_FAILED -eq 0 ]]; then
    echo -e "${GREEN}✓ All test suites passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some test suites failed!${NC}"
    exit 1
fi
