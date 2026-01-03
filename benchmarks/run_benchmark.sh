#!/bin/bash
# Benchmark runner: compares Ninja vs rninja performance
# Usage: run_benchmark.sh [--sizes small,medium,large] [--iterations N] [--output FILE]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Defaults
SIZES="small,medium"
ITERATIONS=3
OUTPUT_FILE="$SCRIPT_DIR/results.json"
RNINJA="$PROJECT_ROOT/target/release/rninja"
NINJA="ninja"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --sizes)
            SIZES="$2"
            shift 2
            ;;
        --iterations)
            ITERATIONS="$2"
            shift 2
            ;;
        --output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --rninja)
            RNINJA="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Check dependencies
if ! command -v $NINJA &> /dev/null; then
    echo "Error: ninja not found in PATH"
    exit 1
fi

if [[ ! -x "$RNINJA" ]]; then
    echo "Building rninja..."
    cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml"
fi

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== rninja Benchmark Suite ===${NC}"
echo ""
echo "Configuration:"
echo "  Sizes: $SIZES"
echo "  Iterations: $ITERATIONS"
echo "  Output: $OUTPUT_FILE"
echo ""

# Function to measure build time
measure_build() {
    local builder="$1"
    local project_dir="$2"
    local iteration="$3"

    cd "$project_dir"

    # Clean first
    rm -rf obj/*.o app 2>/dev/null || true

    # Measure time
    local start_time=$(date +%s.%N)
    $builder > /dev/null 2>&1
    local end_time=$(date +%s.%N)

    local elapsed=$(echo "$end_time - $start_time" | bc)
    echo "$elapsed"

    cd - > /dev/null
}

# Function to measure incremental build time (no-op)
measure_noop() {
    local builder="$1"
    local project_dir="$2"

    cd "$project_dir"

    # Build should be a no-op
    local start_time=$(date +%s.%N)
    $builder > /dev/null 2>&1
    local end_time=$(date +%s.%N)

    local elapsed=$(echo "$end_time - $start_time" | bc)
    echo "$elapsed"

    cd - > /dev/null
}

# Function to measure incremental build after touching one file
measure_incremental() {
    local builder="$1"
    local project_dir="$2"

    cd "$project_dir"

    # Touch one source file
    touch src/file_1.c

    # Measure rebuild
    local start_time=$(date +%s.%N)
    $builder > /dev/null 2>&1
    local end_time=$(date +%s.%N)

    local elapsed=$(echo "$end_time - $start_time" | bc)
    echo "$elapsed"

    cd - > /dev/null
}

# Initialize results
RESULTS="{"
RESULTS+="\"timestamp\": \"$(date -Iseconds)\","
RESULTS+="\"hostname\": \"$(hostname)\","
RESULTS+="\"cpu\": \"$(grep 'model name' /proc/cpuinfo | head -1 | cut -d: -f2 | xargs)\","
RESULTS+="\"cores\": $(nproc),"
RESULTS+="\"iterations\": $ITERATIONS,"
RESULTS+="\"benchmarks\": ["

FIRST_BENCH=true

# Run benchmarks for each size
IFS=',' read -ra SIZE_ARRAY <<< "$SIZES"
for SIZE in "${SIZE_ARRAY[@]}"; do
    echo -e "${YELLOW}=== Benchmarking $SIZE project ===${NC}"

    # Generate project
    PROJECT_DIR="/tmp/rninja-bench-$SIZE"
    "$SCRIPT_DIR/generate_project.sh" "$SIZE" "$PROJECT_DIR" > /dev/null

    TARGET_COUNT=$(grep -c "^build " "$PROJECT_DIR/build.ninja")
    echo "  Targets: $TARGET_COUNT"

    # Arrays to store times
    NINJA_FULL=()
    RNINJA_FULL=()
    NINJA_NOOP=()
    RNINJA_NOOP=()
    NINJA_INCR=()
    RNINJA_INCR=()

    for i in $(seq 1 $ITERATIONS); do
        echo -e "  ${BLUE}Iteration $i/$ITERATIONS${NC}"

        # Full build - Ninja
        time_ninja=$(measure_build "$NINJA" "$PROJECT_DIR" "$i")
        NINJA_FULL+=("$time_ninja")
        echo "    Ninja full: ${time_ninja}s"

        # No-op build - Ninja
        time_ninja_noop=$(measure_noop "$NINJA" "$PROJECT_DIR")
        NINJA_NOOP+=("$time_ninja_noop")
        echo "    Ninja no-op: ${time_ninja_noop}s"

        # Incremental build - Ninja
        time_ninja_incr=$(measure_incremental "$NINJA" "$PROJECT_DIR")
        NINJA_INCR+=("$time_ninja_incr")
        echo "    Ninja incremental: ${time_ninja_incr}s"

        # Clean for rninja
        rm -rf "$PROJECT_DIR/obj/"*.o "$PROJECT_DIR/app" 2>/dev/null || true
        mkdir -p "$PROJECT_DIR/obj"

        # Full build - rninja
        time_rninja=$(measure_build "$RNINJA --no-daemon" "$PROJECT_DIR" "$i")
        RNINJA_FULL+=("$time_rninja")
        echo "    rninja full: ${time_rninja}s"

        # No-op build - rninja
        time_rninja_noop=$(measure_noop "$RNINJA --no-daemon" "$PROJECT_DIR")
        RNINJA_NOOP+=("$time_rninja_noop")
        echo "    rninja no-op: ${time_rninja_noop}s"

        # Incremental build - rninja
        time_rninja_incr=$(measure_incremental "$RNINJA --no-daemon" "$PROJECT_DIR")
        RNINJA_INCR+=("$time_rninja_incr")
        echo "    rninja incremental: ${time_rninja_incr}s"
    done

    # Calculate averages
    calc_avg() {
        local arr=("$@")
        local sum=0
        for val in "${arr[@]}"; do
            sum=$(echo "$sum + $val" | bc)
        done
        echo "scale=4; $sum / ${#arr[@]}" | bc
    }

    NINJA_FULL_AVG=$(calc_avg "${NINJA_FULL[@]}")
    RNINJA_FULL_AVG=$(calc_avg "${RNINJA_FULL[@]}")
    NINJA_NOOP_AVG=$(calc_avg "${NINJA_NOOP[@]}")
    RNINJA_NOOP_AVG=$(calc_avg "${RNINJA_NOOP[@]}")
    NINJA_INCR_AVG=$(calc_avg "${NINJA_INCR[@]}")
    RNINJA_INCR_AVG=$(calc_avg "${RNINJA_INCR[@]}")

    # Calculate speedup
    if (( $(echo "$RNINJA_FULL_AVG > 0" | bc -l) )); then
        FULL_SPEEDUP=$(echo "scale=2; $NINJA_FULL_AVG / $RNINJA_FULL_AVG" | bc)
    else
        FULL_SPEEDUP="N/A"
    fi

    if (( $(echo "$RNINJA_NOOP_AVG > 0" | bc -l) )); then
        NOOP_SPEEDUP=$(echo "scale=2; $NINJA_NOOP_AVG / $RNINJA_NOOP_AVG" | bc)
    else
        NOOP_SPEEDUP="N/A"
    fi

    echo ""
    echo -e "  ${GREEN}Results for $SIZE:${NC}"
    echo "    Full build:   Ninja=${NINJA_FULL_AVG}s, rninja=${RNINJA_FULL_AVG}s (${FULL_SPEEDUP}x)"
    echo "    No-op build:  Ninja=${NINJA_NOOP_AVG}s, rninja=${RNINJA_NOOP_AVG}s (${NOOP_SPEEDUP}x)"
    echo "    Incremental:  Ninja=${NINJA_INCR_AVG}s, rninja=${RNINJA_INCR_AVG}s"
    echo ""

    # Add to JSON
    if [[ "$FIRST_BENCH" != "true" ]]; then
        RESULTS+=","
    fi
    FIRST_BENCH=false

    RESULTS+="{"
    RESULTS+="\"size\": \"$SIZE\","
    RESULTS+="\"targets\": $TARGET_COUNT,"
    RESULTS+="\"ninja\": {"
    RESULTS+="\"full_avg\": $NINJA_FULL_AVG,"
    RESULTS+="\"noop_avg\": $NINJA_NOOP_AVG,"
    RESULTS+="\"incremental_avg\": $NINJA_INCR_AVG"
    RESULTS+="},"
    RESULTS+="\"rninja\": {"
    RESULTS+="\"full_avg\": $RNINJA_FULL_AVG,"
    RESULTS+="\"noop_avg\": $RNINJA_NOOP_AVG,"
    RESULTS+="\"incremental_avg\": $RNINJA_INCR_AVG"
    RESULTS+="}"
    RESULTS+="}"

    # Cleanup
    rm -rf "$PROJECT_DIR"
done

RESULTS+="]}"

# Write results
echo "$RESULTS" | python3 -m json.tool > "$OUTPUT_FILE" 2>/dev/null || echo "$RESULTS" > "$OUTPUT_FILE"

echo -e "${GREEN}Results written to $OUTPUT_FILE${NC}"
