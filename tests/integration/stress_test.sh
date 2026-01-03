#!/bin/bash
# Multi-repo stress and concurrency tests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

RNINJA="$PROJECT_ROOT/target/release/rninja"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Multi-Repo Stress & Concurrency Tests ===${NC}"
echo ""

# Build rninja if needed
if [[ ! -x "$RNINJA" ]]; then
    echo "Building rninja..."
    cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml" 2>/dev/null
fi

PASSED=0
FAILED=0

# Disable cache for stress tests to avoid lock contention
export RNINJA_CACHE_ENABLED=0

# Test 1: Multiple repos building concurrently
echo -e "${YELLOW}Test 1: Multiple repos building concurrently${NC}"

REPO1=$(mktemp -d)
REPO2=$(mktemp -d)
REPO3=$(mktemp -d)

# Create minimal projects
for repo in "$REPO1" "$REPO2" "$REPO3"; do
cat > "$repo/build.ninja" << 'EOF'
rule cc
  command = touch $out
build a.txt: cc
build b.txt: cc
build all: phony a.txt b.txt
default all
EOF
done

# Build all three concurrently
(cd "$REPO1" && $RNINJA --no-daemon > /dev/null 2>&1) &
PID1=$!
(cd "$REPO2" && $RNINJA --no-daemon > /dev/null 2>&1) &
PID2=$!
(cd "$REPO3" && $RNINJA --no-daemon > /dev/null 2>&1) &
PID3=$!

wait $PID1; EXIT1=$?
wait $PID2; EXIT2=$?
wait $PID3; EXIT3=$?

rm -rf "$REPO1" "$REPO2" "$REPO3"

if [[ $EXIT1 -eq 0 ]] && [[ $EXIT2 -eq 0 ]] && [[ $EXIT3 -eq 0 ]]; then
    echo -e "  ${GREEN}PASS${NC}: All three concurrent repo builds completed"
    ((PASSED++))
else
    echo -e "  ${RED}FAIL${NC}: Concurrent repo builds failed"
    ((FAILED++))
fi

# Test 2: Rapid sequential builds
echo -e "${YELLOW}Test 2: Rapid sequential builds (5x)${NC}"

TEST_DIR=$(mktemp -d)
cat > "$TEST_DIR/build.ninja" << 'EOF'
rule cc
  command = touch $out
build a.txt: cc
build b.txt: cc
default a.txt b.txt
EOF

all_passed=true
for i in $(seq 1 5); do
    rm -f "$TEST_DIR/a.txt" 2>/dev/null
    if ! (cd "$TEST_DIR" && $RNINJA --no-daemon > /dev/null 2>&1); then
        all_passed=false
        break
    fi
done

rm -rf "$TEST_DIR"

if $all_passed; then
    echo -e "  ${GREEN}PASS${NC}: All 5 rapid builds completed"
    ((PASSED++))
else
    echo -e "  ${RED}FAIL${NC}: Rapid sequential builds failed"
    ((FAILED++))
fi

# Test 3: High parallelism (-j)
echo -e "${YELLOW}Test 3: High parallelism (-j 32)${NC}"

TEST_DIR=$(mktemp -d)
cat > "$TEST_DIR/build.ninja" << 'EOF'
rule cc
  command = touch $out
build a.txt: cc
build b.txt: cc
build c.txt: cc
default a.txt b.txt c.txt
EOF

if (cd "$TEST_DIR" && $RNINJA --no-daemon -j 32 > /dev/null 2>&1); then
    echo -e "  ${GREEN}PASS${NC}: High parallelism build completed"
    ((PASSED++))
else
    echo -e "  ${RED}FAIL${NC}: High parallelism build failed"
    ((FAILED++))
fi

rm -rf "$TEST_DIR"

# Test 4: Serial build (-j 1)
echo -e "${YELLOW}Test 4: Serial build (-j 1)${NC}"

TEST_DIR=$(mktemp -d)
cat > "$TEST_DIR/build.ninja" << 'EOF'
rule cc
  command = touch $out
build a.txt: cc
build b.txt: cc
default a.txt b.txt
EOF

if (cd "$TEST_DIR" && $RNINJA --no-daemon -j 1 > /dev/null 2>&1); then
    echo -e "  ${GREEN}PASS${NC}: Serial build completed"
    ((PASSED++))
else
    echo -e "  ${RED}FAIL${NC}: Serial build failed"
    ((FAILED++))
fi

rm -rf "$TEST_DIR"

# Test 5: Keep-going on failure
echo -e "${YELLOW}Test 5: Keep-going on failure${NC}"

TEST_DIR=$(mktemp -d)
cat > "$TEST_DIR/build.ninja" << 'EOF'
rule fail
  command = false
rule pass
  command = touch $out
build fail.txt: fail
build pass.txt: pass
default fail.txt pass.txt
EOF

(cd "$TEST_DIR" && $RNINJA --no-daemon -k 0 > /dev/null 2>&1) || true

if [[ -f "$TEST_DIR/pass.txt" ]]; then
    echo -e "  ${GREEN}PASS${NC}: Keep-going built remaining targets"
    ((PASSED++))
else
    echo -e "  ${RED}FAIL${NC}: Keep-going did not build remaining targets"
    ((FAILED++))
fi

rm -rf "$TEST_DIR"

echo ""
echo -e "${BLUE}=== Stress Test Summary ===${NC}"
echo -e "  ${GREEN}Passed${NC}: $PASSED"
echo -e "  ${RED}Failed${NC}: $FAILED"
echo ""

if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}All stress tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some stress tests failed!${NC}"
    exit 1
fi
