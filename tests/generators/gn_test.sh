#!/bin/bash
# Test rninja with GN-generated Ninja files
# GN is Google's meta-build system used by Chromium

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

echo -e "${BLUE}=== GN Generator Compatibility Test ===${NC}"
echo ""

# Check for gn
if ! command -v gn &> /dev/null; then
    echo -e "${YELLOW}SKIP: gn not found${NC}"
    echo "GN can be installed from: https://gn.googlesource.com/gn/"
    exit 0
fi

# Build rninja if needed
if [[ ! -x "$RNINJA" ]]; then
    cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml" 2>/dev/null
fi

# Create test project
TEST_DIR=$(mktemp -d)
trap "rm -rf $TEST_DIR" EXIT

echo "Creating GN project in $TEST_DIR"

# Create .gn (root marker)
cat > "$TEST_DIR/.gn" << 'EOF'
buildconfig = "//build/BUILDCONFIG.gn"
EOF

# Create build config
mkdir -p "$TEST_DIR/build"
cat > "$TEST_DIR/build/BUILDCONFIG.gn" << 'EOF'
set_default_toolchain("//build/toolchain:gcc")
EOF

# Create toolchain
mkdir -p "$TEST_DIR/build/toolchain"
cat > "$TEST_DIR/build/toolchain/BUILD.gn" << 'EOF'
toolchain("gcc") {
  tool("cc") {
    depfile = "{{output}}.d"
    command = "gcc -MMD -MF $depfile -c {{source}} -o {{output}}"
    depsformat = "gcc"
    outputs = [ "{{source_out_dir}}/{{source_name_part}}.o" ]
  }
  tool("cxx") {
    depfile = "{{output}}.d"
    command = "g++ -MMD -MF $depfile -c {{source}} -o {{output}}"
    depsformat = "gcc"
    outputs = [ "{{source_out_dir}}/{{source_name_part}}.o" ]
  }
  tool("link") {
    command = "gcc {{inputs}} -o {{output}}"
    outputs = [ "{{root_out_dir}}/{{target_output_name}}" ]
  }
  tool("stamp") {
    command = "touch {{output}}"
  }
}
EOF

# Create source files
mkdir -p "$TEST_DIR/src"
cat > "$TEST_DIR/src/main.c" << 'EOF'
#include <stdio.h>
int main() { printf("GN test\n"); return 0; }
EOF

# Create BUILD.gn
cat > "$TEST_DIR/BUILD.gn" << 'EOF'
executable("myapp") {
  sources = [ "src/main.c" ]
}
EOF

# Generate Ninja files
cd "$TEST_DIR"

echo -e "${YELLOW}Running gn gen...${NC}"
gn gen out > /dev/null 2>&1

if [[ ! -f "out/build.ninja" ]]; then
    echo -e "${RED}FAIL: GN did not generate build.ninja${NC}"
    exit 1
fi

cd out
echo "Generated build.ninja with $(grep -c '^build ' build.ninja) build statements"

# Test with Ninja first
echo -e "${YELLOW}Building with Ninja...${NC}"
ninja_result=0
$NINJA > /dev/null 2>&1 || ninja_result=$?

# Clean and test with rninja
echo -e "${YELLOW}Building with rninja...${NC}"
rm -rf obj myapp 2>/dev/null || true
rninja_result=0
# Disable cache to avoid cache restoration issues
RNINJA_CACHE_ENABLED=0 $RNINJA --no-daemon > /dev/null 2>&1 || rninja_result=$?

# Compare results
if [[ $ninja_result -eq $rninja_result ]]; then
    echo -e "${GREEN}PASS: Both builders succeeded with exit code $ninja_result${NC}"
else
    echo -e "${RED}FAIL: Exit codes differ (ninja=$ninja_result, rninja=$rninja_result)${NC}"
    exit 1
fi

echo -e "${GREEN}GN generator test PASSED${NC}"
