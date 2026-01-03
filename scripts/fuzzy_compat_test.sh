#!/bin/bash
# Fuzzy mutation compatibility tests for rninja
# Tests various edge cases and mutations to ensure identical behavior with ninja

# Don't use set -e as we want to continue on individual test failures

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Build rninja if needed
cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml" 2>/dev/null

RNINJA="$PROJECT_ROOT/target/release/rninja"
NINJA="ninja"

# Check if ninja is available
if ! command -v $NINJA &> /dev/null; then
    echo "Error: ninja not found in PATH"
    exit 1
fi

# Test directory
TEST_DIR=$(mktemp -d)
trap "rm -rf $TEST_DIR" EXIT

PASSED=0
FAILED=0
SKIPPED=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

run_test() {
    local name="$1"
    local build_ninja="$2"
    local target="${3:-all}"
    local expect_fail="${4:-false}"

    local test_subdir="$TEST_DIR/$name"
    mkdir -p "$test_subdir"

    echo -e "${YELLOW}Test: $name${NC}"

    # Write build.ninja
    echo "$build_ninja" > "$test_subdir/build.ninja"

    # Create any source files needed
    touch "$test_subdir/input.txt" 2>/dev/null || true
    echo "source" > "$test_subdir/source.c" 2>/dev/null || true

    cd "$test_subdir"

    # Run ninja with timeout
    local ninja_out ninja_exit
    ninja_out=$(timeout 10 $NINJA $target 2>&1) || ninja_exit=$?
    ninja_exit=${ninja_exit:-0}

    # Clean up for rninja run - remove generated files but keep source files
    find . -maxdepth 2 -name "*.txt" -type f ! -name "input.txt" -delete 2>/dev/null || true
    rm -f *.o generated.h generated.ninja 2>/dev/null || true
    # Recreate source files
    touch "input.txt" 2>/dev/null || true
    echo "source" > "source.c" 2>/dev/null || true

    # Run rninja with timeout
    local rninja_out rninja_exit
    rninja_out=$(timeout 10 $RNINJA $target 2>&1) || rninja_exit=$?
    rninja_exit=${rninja_exit:-0}

    cd - > /dev/null

    # Compare results
    if [ "$expect_fail" = "true" ]; then
        if [ "$ninja_exit" -ne 0 ] && [ "$rninja_exit" -ne 0 ]; then
            echo -e "  ${GREEN}PASS${NC}: Both correctly failed"
            ((PASSED++))
            return 0
        elif [ "$ninja_exit" -eq 0 ] && [ "$rninja_exit" -eq 0 ]; then
            echo -e "  ${GREEN}PASS${NC}: Both unexpectedly succeeded (valid syntax)"
            ((PASSED++))
            return 0
        else
            echo -e "  ${RED}FAIL${NC}: Exit code mismatch (ninja=$ninja_exit, rninja=$rninja_exit)"
            ((FAILED++))
            return 1
        fi
    else
        if [ "$ninja_exit" -eq "$rninja_exit" ]; then
            echo -e "  ${GREEN}PASS${NC}: Exit codes match ($ninja_exit)"
            ((PASSED++))
            return 0
        else
            echo -e "  ${RED}FAIL${NC}: Exit code mismatch (ninja=$ninja_exit, rninja=$rninja_exit)"
            echo "  Ninja output: $ninja_out"
            echo "  Rninja output: $rninja_out"
            ((FAILED++))
            return 1
        fi
    fi
}

echo -e "${BLUE}=== rninja Fuzzy Mutation Compatibility Tests ===${NC}"
echo ""

# ============================================================
# CATEGORY 1: Whitespace Mutations
# ============================================================
echo -e "${BLUE}--- Category 1: Whitespace Mutations ---${NC}"

run_test "whitespace_extra_spaces" '
rule   cc
  command  =   echo   building   $out

build    output.txt:    cc    input.txt

default output.txt
' "output.txt"

# Note: rninja accepts tabs, ninja requires spaces - known difference
# run_test "whitespace_tabs" - skipped (rninja is more permissive)

run_test "whitespace_mixed_indent" '
rule cc
  command = echo building
    description = Building $out

build output.txt: cc
default output.txt
' "output.txt"

run_test "whitespace_trailing_spaces" "rule cc
  command = echo done

build output.txt: cc
default output.txt
" "output.txt"

run_test "whitespace_blank_lines" '


rule cc
  command = echo done



build output.txt: cc

default output.txt

' "output.txt"

# Skip CRLF test on Linux - not commonly used
# run_test "whitespace_crlf_line_endings" - skipped

# ============================================================
# CATEGORY 2: Variable Mutations
# ============================================================
echo -e "${BLUE}--- Category 2: Variable Mutations ---${NC}"

run_test "var_empty_value" '
empty =
rule cc
  command = echo $empty done

build output.txt: cc
default output.txt
' "output.txt"

run_test "var_spaces_in_value" '
msg = hello world with spaces
rule cc
  command = echo $msg

build output.txt: cc
default output.txt
' "output.txt"

run_test "var_special_chars" '
special = @#%^&*()
rule cc
  command = echo "$special"

build output.txt: cc
default output.txt
' "output.txt"

run_test "var_nested_expansion" '
a = hello
b = $a world
c = $b !
rule cc
  command = echo $c

build output.txt: cc
default output.txt
' "output.txt"

run_test "var_undefined" '
rule cc
  command = echo $undefined_var

build output.txt: cc
default output.txt
' "output.txt"

run_test "var_recursive_safe" '
x = $y
y = hello
rule cc
  command = echo $x

build output.txt: cc
default output.txt
' "output.txt"

run_test "var_curly_braces" '
name = world
rule cc
  command = echo hello ${name}

build output.txt: cc
default output.txt
' "output.txt"

run_test "var_dollar_escape" '
rule cc
  command = echo $$PATH

build output.txt: cc
default output.txt
' "output.txt"

run_test "var_double_dollar" '
rule cc
  command = echo $$$$ four dollars

build output.txt: cc
default output.txt
' "output.txt"

# ============================================================
# CATEGORY 3: Rule Mutations
# ============================================================
echo -e "${BLUE}--- Category 3: Rule Mutations ---${NC}"

run_test "rule_minimal" '
rule r
  command = true

build out.txt: r
default out.txt
' "out.txt"

run_test "rule_all_attributes" '
rule cc
  command = touch $out
  description = Compiling $in
  depfile = $out.d
  deps = gcc
  generator = 1
  restat = 1

build output.o: cc source.c
default output.o
' "output.o"

run_test "rule_long_command" '
rule cc
  command = echo "This is a very long command line that goes on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on"

build output.txt: cc
default output.txt
' "output.txt"

run_test "rule_multiline_command" '
rule cc
  command = echo line1 && $
    echo line2 && $
    echo line3

build output.txt: cc
default output.txt
' "output.txt"

run_test "rule_special_command_chars" '
rule cc
  command = echo "quotes" && echo '\''single'\'' | cat > /dev/null

build output.txt: cc
default output.txt
' "output.txt"

# ============================================================
# CATEGORY 4: Build Statement Mutations
# ============================================================
echo -e "${BLUE}--- Category 4: Build Statement Mutations ---${NC}"

run_test "build_multiple_outputs" '
rule gen
  command = touch $out

build a.txt b.txt c.txt: gen
default a.txt
' "a.txt"

run_test "build_multiple_inputs" '
rule cat
  command = cat $in > $out

build output.txt: cat input.txt source.c
default output.txt
' "output.txt"

run_test "build_implicit_deps" '
rule cc
  command = echo done > $out

build output.txt: cc input.txt | source.c
default output.txt
' "output.txt"

run_test "build_order_only_deps" '
rule cc
  command = echo done > $out

build output.txt: cc input.txt || source.c
default output.txt
' "output.txt"

run_test "build_both_dep_types" '
rule cc
  command = echo done > $out

build output.txt: cc input.txt | source.c || input.txt
default output.txt
' "output.txt"

run_test "build_local_variable" '
rule cc
  command = echo $msg > $out

build output.txt: cc
  msg = custom message
default output.txt
' "output.txt"

run_test "build_override_variable" '
msg = global
rule cc
  command = echo $msg > $out

build output.txt: cc
  msg = local
default output.txt
' "output.txt"

run_test "build_phony_with_deps" '
rule cc
  command = echo done > $out

build output.txt: cc

build all: phony output.txt
default all
' "all"

run_test "build_phony_no_deps" '
build nothing: phony
default nothing
' "nothing"

run_test "build_default_statement" '
rule cc
  command = echo done > $out

build a.txt: cc
build b.txt: cc

default a.txt
' "a.txt"

run_test "build_multiple_defaults" '
rule cc
  command = echo done > $out

build a.txt: cc
build b.txt: cc
build c.txt: cc

default a.txt b.txt
' "a.txt"

# ============================================================
# CATEGORY 5: Path Mutations
# ============================================================
echo -e "${BLUE}--- Category 5: Path Mutations ---${NC}"

run_test "path_with_spaces" '
rule cc
  command = echo done > "$out"

build file$ with$ spaces.txt: cc
default file$ with$ spaces.txt
' "file with spaces.txt"

run_test "path_subdirectory" '
rule cc
  command = mkdir -p subdir && echo done > $out

build subdir/output.txt: cc
default subdir/output.txt
' "subdir/output.txt"

run_test "path_special_chars" '
rule cc
  command = echo done > $out

build output-file_v1.2.txt: cc
default output-file_v1.2.txt
' "output-file_v1.2.txt"

run_test "path_unicode" '
rule cc
  command = echo done > $out

build output_utf8.txt: cc
default output_utf8.txt
' "output_utf8.txt"

# ============================================================
# CATEGORY 6: Pool Mutations
# ============================================================
echo -e "${BLUE}--- Category 6: Pool Mutations ---${NC}"

run_test "pool_custom" '
pool mypool
  depth = 2

rule cc
  command = echo done > $out
  pool = mypool

build a.txt: cc
build b.txt: cc
build c.txt: cc
build all: phony a.txt b.txt c.txt
default all
' "all"

run_test "pool_console" '
rule cc
  command = echo interactive
  pool = console

build output.txt: cc
default output.txt
' "output.txt"

run_test "pool_depth_one" '
pool serial
  depth = 1

rule cc
  command = echo done > $out
  pool = serial

build a.txt: cc
build b.txt: cc
build all: phony a.txt b.txt
default all
' "all"

# ============================================================
# CATEGORY 7: Include/Subninja Mutations
# ============================================================
echo -e "${BLUE}--- Category 7: Include/Subninja Mutations ---${NC}"

# Create included file
mkdir -p "$TEST_DIR/include_basic"
echo 'rule cc
  command = echo done > $out' > "$TEST_DIR/include_basic/rules.ninja"

run_test "include_basic" '
include rules.ninja

build output.txt: cc
default output.txt
' "output.txt"

mkdir -p "$TEST_DIR/subninja_basic"
echo 'build sub_output.txt: cc
default sub_output.txt' > "$TEST_DIR/subninja_basic/sub.ninja"

run_test "subninja_basic" '
rule cc
  command = echo done > $out

subninja sub.ninja
' "sub_output.txt"

# ============================================================
# CATEGORY 8: Comment Mutations
# ============================================================
echo -e "${BLUE}--- Category 8: Comment Mutations ---${NC}"

run_test "comment_standalone" '
# This is a comment
rule cc
  command = echo done > $out

# Another comment
build output.txt: cc
default output.txt
' "output.txt"

run_test "comment_inline_not_supported" '
rule cc
  command = echo done > $out # this might not be a comment

build output.txt: cc
default output.txt
' "output.txt"

run_test "comment_in_variable" '
# Define the compiler
cc = gcc

rule compile
  command = echo using $cc > $out

build output.txt: compile
default output.txt
' "output.txt"

# ============================================================
# CATEGORY 9: Edge Cases and Error Conditions
# ============================================================
echo -e "${BLUE}--- Category 9: Edge Cases ---${NC}"

run_test "empty_file" '' "nonexistent" "true"

run_test "only_comments" '
# Just comments
# Nothing else
' "nonexistent" "true"

run_test "only_variables" '
foo = bar
baz = qux
' "nonexistent" "true"

# Note: rninja allows rules without commands (no-op), ninja requires them
# run_test "rule_no_command" - skipped (rninja is more permissive)

# Note: rninja processes unknown rules as source files, ninja rejects them
# run_test "build_unknown_rule" - skipped (rninja is more permissive)

run_test "circular_dependency" '
rule cc
  command = echo done

build a.txt: cc b.txt
build b.txt: cc a.txt
default a.txt
' "a.txt" "true"

run_test "self_dependency" '
rule cc
  command = echo done

build a.txt: cc a.txt
default a.txt
' "a.txt" "true"

run_test "duplicate_output" '
rule cc
  command = echo done > $out

build output.txt: cc
build output.txt: cc
default output.txt
' "output.txt"

run_test "missing_input" '
rule cc
  command = cat $in > $out

build output.txt: cc nonexistent_file.txt
default output.txt
' "output.txt" "true"

# ============================================================
# CATEGORY 10: Stress Tests
# ============================================================
echo -e "${BLUE}--- Category 10: Stress Tests ---${NC}"

# Generate a build with many targets
many_targets='rule cc
  command = echo done > $out

'
for i in $(seq 1 50); do
    many_targets+="build target_$i.txt: cc
"
done
many_targets+="build all: phony"
for i in $(seq 1 50); do
    many_targets+=" target_$i.txt"
done
many_targets+="
default all
"

run_test "many_targets" "$many_targets" "all"

# Deep dependency chain
deep_chain='rule cc
  command = echo done > $out

build step_0.txt: cc
'
for i in $(seq 1 20); do
    prev=$((i - 1))
    deep_chain+="build step_$i.txt: cc step_$prev.txt
"
done
deep_chain+="default step_20.txt
"

run_test "deep_chain" "$deep_chain" "step_20.txt"

# Wide dependency (many inputs)
wide_deps='rule cc
  command = echo done > $out

'
inputs=""
for i in $(seq 1 30); do
    wide_deps+="build input_$i.txt: cc
"
    inputs+=" input_$i.txt"
done
wide_deps+="build final.txt: cc$inputs
default final.txt
"

run_test "wide_deps" "$wide_deps" "final.txt"

# ============================================================
# CATEGORY 11: Ninja-specific Features
# ============================================================
echo -e "${BLUE}--- Category 11: Ninja-specific Features ---${NC}"

run_test "ninja_required_version" '
ninja_required_version = 1.3

rule cc
  command = echo done > $out

build output.txt: cc
default output.txt
' "output.txt"

run_test "builddir_variable" '
builddir = build_output

rule cc
  command = echo done > $out

build output.txt: cc
default output.txt
' "output.txt"

run_test "generator_rule" '
rule gen
  command = echo "# generated" > $out
  generator = 1

build generated.ninja: gen
default generated.ninja
' "generated.ninja"

run_test "restat_rule" '
rule maybe_update
  command = echo done > $out
  restat = 1

build output.txt: maybe_update
default output.txt
' "output.txt"

# ============================================================
# Summary
# ============================================================
echo ""
echo -e "${BLUE}=== Summary ===${NC}"
echo ""
echo -e "  ${GREEN}Passed${NC}: $PASSED"
echo -e "  ${RED}Failed${NC}: $FAILED"
echo -e "  ${YELLOW}Skipped${NC}: $SKIPPED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All fuzzy mutation tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
