#!/bin/bash
# Generate synthetic build projects for benchmarking
# Usage: generate_project.sh <size> <output_dir>
#   size: small (50 targets), medium (500 targets), large (5000 targets)

set -e

SIZE="${1:-small}"
OUTPUT_DIR="${2:-/tmp/rninja-bench-$SIZE}"

case "$SIZE" in
    small)
        NUM_TARGETS=50
        CHAIN_DEPTH=5
        FANOUT=3
        ;;
    medium)
        NUM_TARGETS=500
        CHAIN_DEPTH=10
        FANOUT=5
        ;;
    large)
        NUM_TARGETS=5000
        CHAIN_DEPTH=20
        FANOUT=10
        ;;
    *)
        echo "Unknown size: $SIZE (use small, medium, or large)"
        exit 1
        ;;
esac

echo "Generating $SIZE project with ~$NUM_TARGETS targets in $OUTPUT_DIR"

rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/src"

# Generate source files
for i in $(seq 1 $NUM_TARGETS); do
    cat > "$OUTPUT_DIR/src/file_$i.c" << EOF
// Source file $i
#include <stdio.h>

void func_$i(void) {
    printf("Function $i\\n");
}
EOF
done

# Generate main.c
cat > "$OUTPUT_DIR/src/main.c" << EOF
// Main file
#include <stdio.h>

int main(void) {
    printf("Hello from benchmark project\\n");
    return 0;
}
EOF

# Generate build.ninja
cat > "$OUTPUT_DIR/build.ninja" << 'EOF'
# Benchmark project build file
ninja_required_version = 1.3

cc = gcc
cflags = -c -O2

rule cc
  command = $cc $cflags -o $out $in
  description = CC $out

rule link
  command = $cc -o $out $in
  description = LINK $out

rule phony
  command = true
  description = PHONY $out

EOF

# Add object file builds
OBJECTS=""
for i in $(seq 1 $NUM_TARGETS); do
    echo "build obj/file_$i.o: cc src/file_$i.c" >> "$OUTPUT_DIR/build.ninja"
    OBJECTS="$OBJECTS obj/file_$i.o"
done

echo "build obj/main.o: cc src/main.c" >> "$OUTPUT_DIR/build.ninja"

# Add link step
echo "" >> "$OUTPUT_DIR/build.ninja"
echo "build app: link obj/main.o $OBJECTS" >> "$OUTPUT_DIR/build.ninja"

# Add default target
echo "" >> "$OUTPUT_DIR/build.ninja"
echo "default app" >> "$OUTPUT_DIR/build.ninja"

# Create obj directory
mkdir -p "$OUTPUT_DIR/obj"

echo "Generated project:"
echo "  Sources: $((NUM_TARGETS + 1))"
echo "  Objects: $((NUM_TARGETS + 1))"
echo "  Output: $OUTPUT_DIR"

# Count actual targets
ACTUAL_TARGETS=$(grep -c "^build " "$OUTPUT_DIR/build.ninja" || echo "0")
echo "  Build statements: $ACTUAL_TARGETS"
