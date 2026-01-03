#!/bin/bash
# Run all integration tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Integration Test Suite ===${NC}"
echo ""

PASSED=0
FAILED=0

run_test() {
    local name="$1"
    local script="$2"

    echo -e "${YELLOW}=== Running $name ===${NC}"
    echo ""

    if bash "$script"; then
        ((PASSED++))
    else
        ((FAILED++))
    fi
    echo ""
}

# Make scripts executable
chmod +x "$SCRIPT_DIR"/*.sh

# Run tests
run_test "Stress Tests" "$SCRIPT_DIR/stress_test.sh"
run_test "Network Tests" "$SCRIPT_DIR/network_test.sh"
run_test "Serialization Tests" "$SCRIPT_DIR/serialization_test.sh"

echo -e "${BLUE}=== Integration Test Summary ===${NC}"
echo -e "  ${GREEN}Passed${NC}: $PASSED"
echo -e "  ${RED}Failed${NC}: $FAILED"
echo ""

if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}All integration tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some integration tests failed!${NC}"
    exit 1
fi
