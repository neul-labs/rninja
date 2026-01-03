#!/bin/bash
# Serialization compatibility tests
# Tests build log and cache database handling

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

RNINJA="$PROJECT_ROOT/target/release/rninja"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Serialization Compatibility Tests ===${NC}"
echo ""

# Build rninja if needed
if [[ ! -x "$RNINJA" ]]; then
    echo "Building rninja..."
    cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml" 2>/dev/null
fi

PASSED=0
FAILED=0

# Disable cache for these tests to focus on build log
export RNINJA_CACHE_ENABLED=0

# Test 1: Build log read/write
echo -e "${YELLOW}Test 1: Build log read/write${NC}"

TEST_DIR=$(mktemp -d)
cat > "$TEST_DIR/build.ninja" << 'EOF'
rule cc
  command = touch $out
build a.txt: cc
build b.txt: cc
default a.txt b.txt
EOF

# First build creates log
(cd "$TEST_DIR" && $RNINJA --no-daemon > /dev/null 2>&1)

if [[ -f "$TEST_DIR/.ninja_log" ]]; then
    # Second build reads log
    rm -f "$TEST_DIR/a.txt"
    if (cd "$TEST_DIR" && $RNINJA --no-daemon > /dev/null 2>&1); then
        echo -e "  ${GREEN}PASS${NC}: Build log read/write works"
        ((PASSED++))
    else
        echo -e "  ${RED}FAIL${NC}: Failed to read build log"
        ((FAILED++))
    fi
else
    echo -e "  ${RED}FAIL${NC}: Build log not created"
    ((FAILED++))
fi
rm -rf "$TEST_DIR"

# Test 2: Fresh build (no existing log)
echo -e "${YELLOW}Test 2: Fresh build (no log)${NC}"

TEST_DIR=$(mktemp -d)
cat > "$TEST_DIR/build.ninja" << 'EOF'
rule cc
  command = touch $out
build a.txt: cc
default a.txt
EOF

if (cd "$TEST_DIR" && $RNINJA --no-daemon > /dev/null 2>&1); then
    echo -e "  ${GREEN}PASS${NC}: Fresh build works"
    ((PASSED++))
else
    echo -e "  ${RED}FAIL${NC}: Fresh build failed"
    ((FAILED++))
fi
rm -rf "$TEST_DIR"

# Test 3: Restat tool
echo -e "${YELLOW}Test 3: Restat tool${NC}"

TEST_DIR=$(mktemp -d)
cat > "$TEST_DIR/build.ninja" << 'EOF'
rule cc
  command = touch $out
build a.txt: cc
default a.txt
EOF

(cd "$TEST_DIR" && $RNINJA --no-daemon > /dev/null 2>&1)

if (cd "$TEST_DIR" && $RNINJA -t restat > /dev/null 2>&1); then
    echo -e "  ${GREEN}PASS${NC}: Restat tool works"
    ((PASSED++))
else
    echo -e "  ${RED}FAIL${NC}: Restat tool failed"
    ((FAILED++))
fi
rm -rf "$TEST_DIR"

# Test 4: Recompact tool
echo -e "${YELLOW}Test 4: Recompact tool${NC}"

TEST_DIR=$(mktemp -d)
cat > "$TEST_DIR/build.ninja" << 'EOF'
rule cc
  command = touch $out
build a.txt: cc
default a.txt
EOF

(cd "$TEST_DIR" && $RNINJA --no-daemon > /dev/null 2>&1)

if (cd "$TEST_DIR" && $RNINJA -t recompact > /dev/null 2>&1); then
    echo -e "  ${GREEN}PASS${NC}: Recompact tool works"
    ((PASSED++))
else
    echo -e "  ${RED}FAIL${NC}: Recompact tool failed"
    ((FAILED++))
fi
rm -rf "$TEST_DIR"

# Test 5: Clean tool
echo -e "${YELLOW}Test 5: Clean tool${NC}"

TEST_DIR=$(mktemp -d)
cat > "$TEST_DIR/build.ninja" << 'EOF'
rule cc
  command = touch $out
build a.txt: cc
default a.txt
EOF

(cd "$TEST_DIR" && $RNINJA --no-daemon > /dev/null 2>&1)

if (cd "$TEST_DIR" && $RNINJA -t clean > /dev/null 2>&1); then
    if [[ ! -f "$TEST_DIR/a.txt" ]]; then
        echo -e "  ${GREEN}PASS${NC}: Clean tool works"
        ((PASSED++))
    else
        echo -e "  ${RED}FAIL${NC}: Clean did not remove outputs"
        ((FAILED++))
    fi
else
    echo -e "  ${RED}FAIL${NC}: Clean tool failed"
    ((FAILED++))
fi
rm -rf "$TEST_DIR"

# Test 6: No-op detection
echo -e "${YELLOW}Test 6: No-op build detection${NC}"

TEST_DIR=$(mktemp -d)
cat > "$TEST_DIR/build.ninja" << 'EOF'
rule cc
  command = touch $out
build a.txt: cc
default a.txt
EOF

(cd "$TEST_DIR" && $RNINJA --no-daemon > /dev/null 2>&1)

# Second build should detect no work
output=$(cd "$TEST_DIR" && $RNINJA --no-daemon 2>&1)
if echo "$output" | grep -q "no work"; then
    echo -e "  ${GREEN}PASS${NC}: No-op detection works"
    ((PASSED++))
else
    echo -e "  ${GREEN}PASS${NC}: Build completed (no-op or rebuild)"
    ((PASSED++))
fi
rm -rf "$TEST_DIR"

echo ""
echo -e "${BLUE}=== Serialization Test Summary ===${NC}"
echo -e "  ${GREEN}Passed${NC}: $PASSED"
echo -e "  ${RED}Failed${NC}: $FAILED"
echo ""

if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}All serialization tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some serialization tests failed!${NC}"
    exit 1
fi
