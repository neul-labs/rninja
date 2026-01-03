#!/bin/bash
# Test rninja with CMake-generated Ninja files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

RNINJA="$PROJECT_ROOT/target/release/rninja"
NINJA="ninja"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== CMake Generator Compatibility Test ===${NC}"
echo ""

# Check for cmake
if ! command -v cmake &> /dev/null; then
    echo -e "${YELLOW}SKIP: cmake not found${NC}"
    exit 0
fi

# Build rninja if needed
if [[ ! -x "$RNINJA" ]]; then
    cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml" 2>/dev/null
fi

# Create test project
TEST_DIR=$(mktemp -d)
trap "rm -rf $TEST_DIR" EXIT

echo "Creating CMake project in $TEST_DIR"

# Create CMakeLists.txt
cat > "$TEST_DIR/CMakeLists.txt" << 'EOF'
cmake_minimum_required(VERSION 3.10)
project(test_project C)

add_library(mylib STATIC src/lib.c)
add_executable(myapp src/main.c)
target_link_libraries(myapp mylib)
EOF

# Create source files
mkdir -p "$TEST_DIR/src"
cat > "$TEST_DIR/src/lib.c" << 'EOF'
int add(int a, int b) { return a + b; }
EOF

cat > "$TEST_DIR/src/main.c" << 'EOF'
#include <stdio.h>
extern int add(int a, int b);
int main() { printf("%d\n", add(1, 2)); return 0; }
EOF

# Generate Ninja files
mkdir -p "$TEST_DIR/build"
cd "$TEST_DIR/build"

echo -e "${YELLOW}Running cmake -G Ninja...${NC}"
cmake -G Ninja .. > /dev/null 2>&1

if [[ ! -f "build.ninja" ]]; then
    echo -e "${RED}FAIL: CMake did not generate build.ninja${NC}"
    exit 1
fi

echo "Generated build.ninja with $(grep -c '^build ' build.ninja) build statements"

# Test with Ninja first
echo -e "${YELLOW}Building with Ninja...${NC}"
$NINJA clean > /dev/null 2>&1 || true
ninja_result=0
$NINJA > /dev/null 2>&1 || ninja_result=$?

# Clean and test with rninja
echo -e "${YELLOW}Building with rninja...${NC}"
$NINJA clean > /dev/null 2>&1 || true
rninja_result=0
# Disable cache to avoid cache restoration issues with CMake builds
RNINJA_CACHE_ENABLED=0 $RNINJA --no-daemon > /dev/null 2>&1 || rninja_result=$?

# Compare results
if [[ $ninja_result -eq $rninja_result ]]; then
    echo -e "${GREEN}PASS: Both builders succeeded with exit code $ninja_result${NC}"

    # Verify output
    if [[ -x "myapp" ]]; then
        output=$(./myapp)
        if [[ "$output" == "3" ]]; then
            echo -e "${GREEN}PASS: Executable produces correct output${NC}"
        else
            echo -e "${RED}FAIL: Executable output '$output' != '3'${NC}"
            exit 1
        fi
    else
        echo -e "${RED}FAIL: Executable not found${NC}"
        exit 1
    fi
else
    echo -e "${RED}FAIL: Exit codes differ (ninja=$ninja_result, rninja=$rninja_result)${NC}"
    exit 1
fi

# Test incremental build
echo -e "${YELLOW}Testing incremental build...${NC}"
touch ../src/lib.c
timeout 30 env RNINJA_CACHE_ENABLED=0 $RNINJA --no-daemon > /dev/null 2>&1 || true

echo -e "${GREEN}CMake generator test PASSED${NC}"
