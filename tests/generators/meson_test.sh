#!/bin/bash
# Test rninja with Meson-generated Ninja files

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

echo -e "${BLUE}=== Meson Generator Compatibility Test ===${NC}"
echo ""

# Check for meson
if ! command -v meson &> /dev/null; then
    echo -e "${YELLOW}SKIP: meson not found${NC}"
    exit 0
fi

# Build rninja if needed
if [[ ! -x "$RNINJA" ]]; then
    cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml" 2>/dev/null
fi

# Create test project
TEST_DIR=$(mktemp -d)
trap "rm -rf $TEST_DIR" EXIT

echo "Creating Meson project in $TEST_DIR"

# Create meson.build
cat > "$TEST_DIR/meson.build" << 'EOF'
project('test_project', 'c')

lib = static_library('mylib', 'src/lib.c')
executable('myapp', 'src/main.c', link_with: lib)
EOF

# Create source files
mkdir -p "$TEST_DIR/src"
cat > "$TEST_DIR/src/lib.c" << 'EOF'
int multiply(int a, int b) { return a * b; }
EOF

cat > "$TEST_DIR/src/main.c" << 'EOF'
#include <stdio.h>
extern int multiply(int a, int b);
int main() { printf("%d\n", multiply(6, 7)); return 0; }
EOF

# Generate Ninja files
cd "$TEST_DIR"

echo -e "${YELLOW}Running meson setup...${NC}"
meson setup build > /dev/null 2>&1

if [[ ! -f "build/build.ninja" ]]; then
    echo -e "${RED}FAIL: Meson did not generate build.ninja${NC}"
    exit 1
fi

cd build
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
# Disable cache to avoid cache restoration issues
RNINJA_CACHE_ENABLED=0 $RNINJA --no-daemon > /dev/null 2>&1 || rninja_result=$?

# Compare results
if [[ $ninja_result -eq $rninja_result ]]; then
    echo -e "${GREEN}PASS: Both builders succeeded with exit code $ninja_result${NC}"

    # Verify output
    if [[ -x "myapp" ]]; then
        output=$(./myapp)
        if [[ "$output" == "42" ]]; then
            echo -e "${GREEN}PASS: Executable produces correct output${NC}"
        else
            echo -e "${RED}FAIL: Executable output '$output' != '42'${NC}"
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

echo -e "${GREEN}Meson generator test PASSED${NC}"
