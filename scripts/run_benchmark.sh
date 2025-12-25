#!/bin/bash
#
# Run benchmarks comparing ninja vs rninja
#
# Usage:
#   ./run_benchmark.sh [num_files] [num_runs]
#
# Arguments:
#   num_files - Number of C source files for benchmark (default: 100)
#   num_runs  - Number of runs for averaging (default: 5)
#
# Requirements:
#   - ninja (system)
#   - rninja (built from this repo)
#   - gcc
#   - python3

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RNINJA="$PROJECT_DIR/target/release/rninja"

NUM_FILES="${1:-100}"
NUM_RUNS="${2:-5}"
BENCH_DIR="/tmp/rninja_bench_$$"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== rninja Benchmark ===${NC}"
echo ""

# Check prerequisites
if ! command -v ninja &> /dev/null; then
    echo -e "${RED}Error: ninja not found in PATH${NC}"
    exit 1
fi

if ! command -v gcc &> /dev/null; then
    echo -e "${RED}Error: gcc not found in PATH${NC}"
    exit 1
fi

if [ ! -x "$RNINJA" ]; then
    echo -e "${YELLOW}Building rninja...${NC}"
    (cd "$PROJECT_DIR" && cargo build --release)
fi

# Create benchmark project
echo -e "${YELLOW}Generating benchmark project with $NUM_FILES files...${NC}"
mkdir -p "$BENCH_DIR"
uv run "$SCRIPT_DIR/gen_bench.py" "$NUM_FILES" "$BENCH_DIR"
cd "$BENCH_DIR"

echo ""
echo -e "${BLUE}=== Full Build Benchmark ===${NC}"
echo ""

# Benchmark ninja full builds
echo -e "${GREEN}ninja full build ($NUM_RUNS runs):${NC}"
ninja_times=()
for i in $(seq 1 $NUM_RUNS); do
    rm -f *.o program .ninja_log 2>/dev/null
    t=$( { /usr/bin/time -f "%e" ninja 2>&1 1>/dev/null; } 2>&1 )
    ninja_times+=("$t")
    echo "  Run $i: ${t}s"
done

# Calculate ninja average
ninja_avg=$(echo "${ninja_times[@]}" | tr ' ' '\n' | awk '{sum+=$1} END {printf "%.2f", sum/NR}')
echo -e "  ${YELLOW}Average: ${ninja_avg}s${NC}"

echo ""

# Benchmark rninja full builds
echo -e "${GREEN}rninja full build ($NUM_RUNS runs):${NC}"
rninja_times=()
for i in $(seq 1 $NUM_RUNS); do
    rm -f *.o program .ninja_log 2>/dev/null
    t=$( { /usr/bin/time -f "%e" "$RNINJA" 2>&1 1>/dev/null; } 2>&1 )
    rninja_times+=("$t")
    echo "  Run $i: ${t}s"
done

# Calculate rninja average
rninja_avg=$(echo "${rninja_times[@]}" | tr ' ' '\n' | awk '{sum+=$1} END {printf "%.2f", sum/NR}')
echo -e "  ${YELLOW}Average: ${rninja_avg}s${NC}"

# Calculate speedup
speedup=$(echo "$ninja_avg $rninja_avg" | awk '{printf "%.1f", $1/$2}')
echo ""
echo -e "${BLUE}Full build speedup: ${speedup}x${NC}"

echo ""
echo -e "${BLUE}=== No-op Build Benchmark ===${NC}"
echo ""

# Build once to have everything up-to-date
ninja >/dev/null 2>&1

# Benchmark ninja no-op
echo -e "${GREEN}ninja no-op ($NUM_RUNS runs):${NC}"
ninja_noop_times=()
for i in $(seq 1 $NUM_RUNS); do
    t=$( { /usr/bin/time -f "%e" ninja 2>&1 1>/dev/null; } 2>&1 )
    ninja_noop_times+=("$t")
    echo "  Run $i: ${t}s"
done

ninja_noop_avg=$(echo "${ninja_noop_times[@]}" | tr ' ' '\n' | awk '{sum+=$1} END {printf "%.3f", sum/NR}')
echo -e "  ${YELLOW}Average: ${ninja_noop_avg}s${NC}"

echo ""

# Build with rninja to create its log
"$RNINJA" >/dev/null 2>&1

# Benchmark rninja no-op
echo -e "${GREEN}rninja no-op ($NUM_RUNS runs):${NC}"
rninja_noop_times=()
for i in $(seq 1 $NUM_RUNS); do
    t=$( { /usr/bin/time -f "%e" "$RNINJA" 2>&1 1>/dev/null; } 2>&1 )
    rninja_noop_times+=("$t")
    echo "  Run $i: ${t}s"
done

rninja_noop_avg=$(echo "${rninja_noop_times[@]}" | tr ' ' '\n' | awk '{sum+=$1} END {printf "%.3f", sum/NR}')
echo -e "  ${YELLOW}Average: ${rninja_noop_avg}s${NC}"

# Calculate no-op speedup
noop_speedup=$(echo "$ninja_noop_avg $rninja_noop_avg" | awk '{if($2>0) printf "%.1f", $1/$2; else print "inf"}')
echo ""
echo -e "${BLUE}No-op speedup: ${noop_speedup}x${NC}"

echo ""
echo -e "${BLUE}=== Cache Hit Benchmark ===${NC}"
echo ""

# Clean and build with cache
rm -f *.o program .ninja_log
echo "Populating cache..."
"$RNINJA" >/dev/null 2>&1

# Clean outputs but keep cache
rm -f *.o program .ninja_log

echo -e "${GREEN}rninja rebuild with cache ($NUM_RUNS runs):${NC}"
cache_times=()
for i in $(seq 1 $NUM_RUNS); do
    rm -f *.o program .ninja_log 2>/dev/null
    t=$( { /usr/bin/time -f "%e" "$RNINJA" 2>&1 1>/dev/null; } 2>&1 )
    cache_times+=("$t")
    echo "  Run $i: ${t}s"
done

cache_avg=$(echo "${cache_times[@]}" | tr ' ' '\n' | awk '{sum+=$1} END {printf "%.2f", sum/NR}')
echo -e "  ${YELLOW}Average: ${cache_avg}s${NC}"

cache_speedup=$(echo "$ninja_avg $cache_avg" | awk '{if($2>0) printf "%.1f", $1/$2; else print "inf"}')
echo ""
echo -e "${BLUE}Cache hit speedup vs ninja cold build: ${cache_speedup}x${NC}"

# Cleanup
echo ""
echo -e "${YELLOW}Cleaning up benchmark directory...${NC}"
rm -rf "$BENCH_DIR"

echo ""
echo -e "${BLUE}=== Summary ===${NC}"
echo ""
echo "Test: $NUM_FILES C files, $NUM_RUNS runs each"
echo ""
echo "| Scenario | ninja | rninja | Speedup |"
echo "|----------|-------|--------|---------|"
echo "| Full build | ${ninja_avg}s | ${rninja_avg}s | ${speedup}x |"
echo "| No-op build | ${ninja_noop_avg}s | ${rninja_noop_avg}s | ${noop_speedup}x |"
echo "| Cached rebuild | - | ${cache_avg}s | ${cache_speedup}x vs ninja |"
echo ""
