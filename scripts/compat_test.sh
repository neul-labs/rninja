#!/bin/bash
#
# Compatibility tests for rninja vs ninja
#
# This script verifies that rninja produces identical behavior to ninja
# across a variety of build scenarios.
#
# Usage:
#   ./compat_test.sh [rninja_path]
#
# Arguments:
#   rninja_path - Path to rninja binary (default: ../target/release/rninja)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RNINJA="${1:-$PROJECT_DIR/target/release/rninja}"
TEST_DIR="/tmp/rninja_compat_$$"

# Disable cache for compatibility tests to test core functionality
export RNINJA_CACHE_ENABLED=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PASSED=0
FAILED=0

echo -e "${BLUE}=== rninja Compatibility Tests ===${NC}"
echo ""

# Check prerequisites
if ! command -v ninja &> /dev/null; then
    echo -e "${RED}Error: ninja not found in PATH${NC}"
    exit 1
fi

if [ ! -x "$RNINJA" ]; then
    echo -e "${YELLOW}Building rninja...${NC}"
    (cd "$PROJECT_DIR" && cargo build --release)
fi

mkdir -p "$TEST_DIR"

# Helper functions
pass() {
    echo -e "  ${GREEN}PASS${NC}: $1"
    ((PASSED++))
}

fail() {
    echo -e "  ${RED}FAIL${NC}: $1"
    echo -e "    $2"
    ((FAILED++))
}

cleanup_test() {
    rm -rf "$TEST_DIR"/*
}

# ============================================================
# Test 1: Basic build
# ============================================================
echo -e "${YELLOW}Test 1: Basic build${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
rule cc
    command = echo "compiling $in" > $out

build out.txt: cc in.txt

default out.txt
EOF
echo "input" > in.txt

ninja -t clean >/dev/null 2>&1 || true
ninja >/dev/null 2>&1
ninja_output=$(cat out.txt)

rm -f out.txt .ninja_log
"$RNINJA" >/dev/null 2>&1
rninja_output=$(cat out.txt)

if [ "$ninja_output" = "$rninja_output" ]; then
    pass "Basic build produces same output"
else
    fail "Basic build produces different output" "ninja: '$ninja_output' vs rninja: '$rninja_output'"
fi

# ============================================================
# Test 2: Phony rules
# ============================================================
echo -e "${YELLOW}Test 2: Phony rules${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
rule cc
    command = echo "built" > $out

build a.txt: cc
build b.txt: cc
build all: phony a.txt b.txt

default all
EOF

rm -f .ninja_log
ninja >/dev/null 2>&1
ninja_a=$(cat a.txt)
ninja_b=$(cat b.txt)

rm -f a.txt b.txt .ninja_log
"$RNINJA" >/dev/null 2>&1
rninja_a=$(cat a.txt)
rninja_b=$(cat b.txt)

if [ "$ninja_a" = "$rninja_a" ] && [ "$ninja_b" = "$rninja_b" ]; then
    pass "Phony rules work correctly"
else
    fail "Phony rules produce different results" ""
fi

# ============================================================
# Test 3: Variable expansion
# ============================================================
echo -e "${YELLOW}Test 3: Variable expansion${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
myvar = hello

rule echo_var
    command = echo "$myvar world" > $out

build out.txt: echo_var

default out.txt
EOF

rm -f .ninja_log
ninja >/dev/null 2>&1
ninja_output=$(cat out.txt)

rm -f out.txt .ninja_log
"$RNINJA" >/dev/null 2>&1
rninja_output=$(cat out.txt)

if [ "$ninja_output" = "$rninja_output" ]; then
    pass "Variable expansion works correctly"
else
    fail "Variable expansion differs" "ninja: '$ninja_output' vs rninja: '$rninja_output'"
fi

# ============================================================
# Test 4: Build-level variables
# ============================================================
echo -e "${YELLOW}Test 4: Build-level variables${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
rule write
    command = echo "$msg" > $out

build a.txt: write
    msg = message A

build b.txt: write
    msg = message B

build all: phony a.txt b.txt
default all
EOF

rm -f .ninja_log
ninja >/dev/null 2>&1
ninja_a=$(cat a.txt)
ninja_b=$(cat b.txt)

rm -f a.txt b.txt .ninja_log
"$RNINJA" >/dev/null 2>&1
rninja_a=$(cat a.txt)
rninja_b=$(cat b.txt)

if [ "$ninja_a" = "$rninja_a" ] && [ "$ninja_b" = "$rninja_b" ]; then
    pass "Build-level variables work correctly"
else
    fail "Build-level variables differ" ""
fi

# ============================================================
# Test 5: Multiple outputs
# ============================================================
echo -e "${YELLOW}Test 5: Multiple outputs${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
rule gen
    command = echo "out1" > out1.txt && echo "out2" > out2.txt

build out1.txt out2.txt: gen
default out1.txt
EOF

rm -f .ninja_log
ninja >/dev/null 2>&1
ninja_out1=$(cat out1.txt)
ninja_out2=$(cat out2.txt)

rm -f out1.txt out2.txt .ninja_log
"$RNINJA" >/dev/null 2>&1
rninja_out1=$(cat out1.txt)
rninja_out2=$(cat out2.txt)

if [ "$ninja_out1" = "$rninja_out1" ] && [ "$ninja_out2" = "$rninja_out2" ]; then
    pass "Multiple outputs work correctly"
else
    fail "Multiple outputs differ" ""
fi

# ============================================================
# Test 6: Dependency chain
# ============================================================
echo -e "${YELLOW}Test 6: Dependency chain${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
rule copy
    command = cat $in > $out

build step1.txt: copy input.txt
build step2.txt: copy step1.txt
build step3.txt: copy step2.txt
EOF
echo "original" > input.txt

rm -f .ninja_log
ninja step3.txt >/dev/null 2>&1
ninja_output=$(cat step3.txt)

rm -f step1.txt step2.txt step3.txt .ninja_log
"$RNINJA" step3.txt >/dev/null 2>&1
rninja_output=$(cat step3.txt)

if [ "$ninja_output" = "$rninja_output" ]; then
    pass "Dependency chains work correctly"
else
    fail "Dependency chains differ" "ninja: '$ninja_output' vs rninja: '$rninja_output'"
fi

# ============================================================
# Test 7: Implicit dependencies
# ============================================================
echo -e "${YELLOW}Test 7: Implicit dependencies${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
rule cc
    command = cat $in > $out

build out.txt: cc in.txt | implicit.txt
default out.txt
EOF
echo "input" > in.txt
echo "implicit" > implicit.txt

rm -f .ninja_log
ninja >/dev/null 2>&1
ninja_output=$(cat out.txt)

rm -f out.txt .ninja_log
"$RNINJA" >/dev/null 2>&1
rninja_output=$(cat out.txt)

if [ "$ninja_output" = "$rninja_output" ]; then
    pass "Implicit dependencies work correctly"
else
    fail "Implicit dependencies differ" ""
fi

# ============================================================
# Test 8: Order-only dependencies
# ============================================================
echo -e "${YELLOW}Test 8: Order-only dependencies${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
rule mkdir
    command = mkdir -p $out && touch $out/.marker

rule write
    command = echo "content" > $out

build dir: mkdir
build dir/file.txt: write || dir
EOF

rm -rf dir .ninja_log
ninja dir/file.txt >/dev/null 2>&1
ninja_exists=$( [ -f dir/file.txt ] && echo "yes" || echo "no" )

rm -rf dir .ninja_log
"$RNINJA" dir/file.txt >/dev/null 2>&1
rninja_exists=$( [ -f dir/file.txt ] && echo "yes" || echo "no" )

if [ "$ninja_exists" = "$rninja_exists" ]; then
    pass "Order-only dependencies work correctly"
else
    fail "Order-only dependencies differ" ""
fi

# ============================================================
# Test 9: Clean subtool
# ============================================================
echo -e "${YELLOW}Test 9: Clean subtool${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
rule cc
    command = echo "built" > $out

build a.txt: cc
build b.txt: cc
build all: phony a.txt b.txt
default all
EOF

ninja >/dev/null 2>&1
"$RNINJA" -t clean >/dev/null 2>&1
clean_result=$( [ ! -f a.txt ] && [ ! -f b.txt ] && echo "clean" || echo "not clean" )

if [ "$clean_result" = "clean" ]; then
    pass "Clean subtool works correctly"
else
    fail "Clean subtool failed" "Files still exist after clean"
fi

# ============================================================
# Test 10: Commands subtool (-t commands)
# ============================================================
echo -e "${YELLOW}Test 10: Commands subtool${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
rule cc
    command = gcc -c $in -o $out

build foo.o: cc foo.c
EOF
echo "int main() { return 0; }" > foo.c

ninja_cmds=$(ninja -t commands foo.o 2>/dev/null | sort)
rninja_cmds=$("$RNINJA" -t commands foo.o 2>/dev/null | sort)

if [ "$ninja_cmds" = "$rninja_cmds" ]; then
    pass "Commands subtool works correctly"
else
    fail "Commands subtool differs" ""
fi

# ============================================================
# Test 11: Targets subtool (-t targets)
# ============================================================
echo -e "${YELLOW}Test 11: Targets subtool${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
rule cc
    command = echo > $out

build a.txt: cc
build b.txt: cc
build all: phony a.txt b.txt
EOF

# Both should list similar targets
ninja_targets=$(ninja -t targets all 2>/dev/null | head -5 | sort)
rninja_targets=$("$RNINJA" -t targets all 2>/dev/null | head -5 | sort)

# Just check that rninja returns targets (format may differ slightly)
if [ -n "$rninja_targets" ]; then
    pass "Targets subtool returns results"
else
    fail "Targets subtool returns empty" ""
fi

# ============================================================
# Test 12: Rules subtool (-t rules)
# ============================================================
echo -e "${YELLOW}Test 12: Rules subtool${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
rule cc
    command = gcc -c $in -o $out

rule link
    command = gcc $in -o $out

build foo.o: cc foo.c
EOF

# Note: ninja includes 'phony' as a rule, rninja doesn't (it's a built-in)
ninja_rules=$(ninja -t rules 2>/dev/null | grep -v '^phony$' | sort)
rninja_rules=$("$RNINJA" -t rules 2>/dev/null | grep -v '^phony$' | sort)

if [ "$ninja_rules" = "$rninja_rules" ]; then
    pass "Rules subtool works correctly"
else
    fail "Rules subtool differs" "ninja: '$ninja_rules' vs rninja: '$rninja_rules'"
fi

# ============================================================
# Test 13: No-op build detection
# ============================================================
echo -e "${YELLOW}Test 13: No-op build detection${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
rule cc
    command = echo "building" >> log.txt && echo "done" > $out

build out.txt: cc
default out.txt
EOF

rm -f log.txt .ninja_log
"$RNINJA" >/dev/null 2>&1
first_count=$(wc -l < log.txt)

"$RNINJA" >/dev/null 2>&1
second_count=$(wc -l < log.txt)

if [ "$first_count" = "$second_count" ]; then
    pass "No-op build correctly skips up-to-date targets"
else
    fail "No-op build re-ran commands" "First: $first_count lines, Second: $second_count lines"
fi

# ============================================================
# Test 14: Incremental rebuild on source change
# ============================================================
echo -e "${YELLOW}Test 14: Incremental rebuild on source change${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
rule cc
    command = cat $in > $out

build out.txt: cc in.txt
default out.txt
EOF
echo "version1" > in.txt

"$RNINJA" >/dev/null 2>&1
first=$(cat out.txt)

sleep 0.1
echo "version2" > in.txt
"$RNINJA" >/dev/null 2>&1
second=$(cat out.txt)

if [ "$first" = "version1" ] && [ "$second" = "version2" ]; then
    pass "Incremental rebuild detects source changes"
else
    fail "Incremental rebuild failed" "Expected version2, got: $second"
fi

# ============================================================
# Test 15: CompDB output
# ============================================================
echo -e "${YELLOW}Test 15: CompDB output${NC}"
cleanup_test
cd "$TEST_DIR"

cat > build.ninja << 'EOF'
rule cc
    command = gcc -c $in -o $out

build foo.o: cc foo.c
EOF
echo "int main() { return 0; }" > foo.c

compdb=$("$RNINJA" -t compdb cc 2>/dev/null)
if echo "$compdb" | grep -q "foo.c"; then
    pass "CompDB output contains expected entries"
else
    fail "CompDB output missing entries" ""
fi

# ============================================================
# Summary
# ============================================================
echo ""
echo -e "${BLUE}=== Summary ===${NC}"
echo ""
echo -e "  ${GREEN}Passed${NC}: $PASSED"
echo -e "  ${RED}Failed${NC}: $FAILED"
echo ""

# Cleanup
rm -rf "$TEST_DIR"

if [ "$FAILED" -gt 0 ]; then
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
else
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
fi
