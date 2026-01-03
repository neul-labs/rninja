#!/bin/bash
# Run all generator compatibility tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Generator Compatibility Test Suite ===${NC}"
echo ""

PASSED=0
FAILED=0
SKIPPED=0

run_test() {
    local name="$1"
    local script="$2"

    echo -e "${YELLOW}Running $name test...${NC}"

    if bash "$script"; then
        ((PASSED++))
        echo ""
    else
        exit_code=$?
        if [[ $exit_code -eq 0 ]]; then
            ((SKIPPED++))
        else
            ((FAILED++))
        fi
        echo ""
    fi
}

# Make scripts executable
chmod +x "$SCRIPT_DIR"/*.sh

# Run tests
run_test "CMake" "$SCRIPT_DIR/cmake_test.sh"
run_test "Meson" "$SCRIPT_DIR/meson_test.sh"
run_test "GN" "$SCRIPT_DIR/gn_test.sh"

echo -e "${BLUE}=== Generator Test Summary ===${NC}"
echo -e "  ${GREEN}Passed${NC}:  $PASSED"
echo -e "  ${RED}Failed${NC}:  $FAILED"
echo -e "  ${YELLOW}Skipped${NC}: $SKIPPED"
echo ""

if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}All available generator tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
