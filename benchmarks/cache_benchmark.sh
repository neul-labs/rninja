#!/bin/bash
# Cache performance benchmark
# Measures cache hit/miss performance and speedup from caching

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

RNINJA="$PROJECT_ROOT/target/release/rninja"
SIZE="${1:-medium}"
ITERATIONS="${2:-3}"
OUTPUT_FILE="$SCRIPT_DIR/cache_results.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== rninja Cache Benchmark ===${NC}"
echo ""

# Build rninja if needed
if [[ ! -x "$RNINJA" ]]; then
    echo "Building rninja..."
    cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml"
fi

# Generate project
PROJECT_DIR="/tmp/rninja-cache-bench"
"$SCRIPT_DIR/generate_project.sh" "$SIZE" "$PROJECT_DIR" > /dev/null

TARGET_COUNT=$(grep -c "^build " "$PROJECT_DIR/build.ninja")
echo "Project size: $SIZE ($TARGET_COUNT targets)"
echo "Iterations: $ITERATIONS"
echo ""

# Set up cache directory
CACHE_DIR="/tmp/rninja-cache-bench-cache"
rm -rf "$CACHE_DIR"
mkdir -p "$CACHE_DIR"
export RNINJA_CACHE_DIR="$CACHE_DIR"
export RNINJA_CACHE_ENABLED=1

cd "$PROJECT_DIR"

# Arrays for results
COLD_TIMES=()
WARM_TIMES=()
NOCACHE_TIMES=()

echo -e "${YELLOW}=== Cold Cache Builds ===${NC}"
for i in $(seq 1 $ITERATIONS); do
    # Clean everything
    rm -rf obj/*.o app "$CACHE_DIR"/* 2>/dev/null || true
    mkdir -p obj

    start_time=$(date +%s.%N)
    $RNINJA --no-daemon 2>/dev/null
    end_time=$(date +%s.%N)

    elapsed=$(echo "$end_time - $start_time" | bc)
    COLD_TIMES+=("$elapsed")
    echo "  Iteration $i: ${elapsed}s (cold cache)"
done

echo ""
echo -e "${YELLOW}=== Warm Cache Builds (clean + rebuild) ===${NC}"
for i in $(seq 1 $ITERATIONS); do
    # Clean build artifacts but keep cache
    rm -rf obj/*.o app 2>/dev/null || true
    mkdir -p obj

    start_time=$(date +%s.%N)
    $RNINJA --no-daemon 2>/dev/null
    end_time=$(date +%s.%N)

    elapsed=$(echo "$end_time - $start_time" | bc)
    WARM_TIMES+=("$elapsed")
    echo "  Iteration $i: ${elapsed}s (warm cache)"
done

echo ""
echo -e "${YELLOW}=== No Cache Builds ===${NC}"
export RNINJA_CACHE_ENABLED=0
for i in $(seq 1 $ITERATIONS); do
    # Clean everything
    rm -rf obj/*.o app 2>/dev/null || true
    mkdir -p obj

    start_time=$(date +%s.%N)
    $RNINJA --no-daemon 2>/dev/null
    end_time=$(date +%s.%N)

    elapsed=$(echo "$end_time - $start_time" | bc)
    NOCACHE_TIMES+=("$elapsed")
    echo "  Iteration $i: ${elapsed}s (no cache)"
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

COLD_AVG=$(calc_avg "${COLD_TIMES[@]}")
WARM_AVG=$(calc_avg "${WARM_TIMES[@]}")
NOCACHE_AVG=$(calc_avg "${NOCACHE_TIMES[@]}")

# Calculate speedups
if (( $(echo "$WARM_AVG > 0" | bc -l) )); then
    CACHE_SPEEDUP=$(echo "scale=2; $COLD_AVG / $WARM_AVG" | bc)
else
    CACHE_SPEEDUP="N/A"
fi

if (( $(echo "$WARM_AVG > 0" | bc -l) )); then
    NOCACHE_SPEEDUP=$(echo "scale=2; $NOCACHE_AVG / $WARM_AVG" | bc)
else
    NOCACHE_SPEEDUP="N/A"
fi

echo ""
echo -e "${GREEN}=== Cache Benchmark Results ===${NC}"
echo "  Cold cache avg:  ${COLD_AVG}s"
echo "  Warm cache avg:  ${WARM_AVG}s"
echo "  No cache avg:    ${NOCACHE_AVG}s"
echo ""
echo "  Cache speedup (cold -> warm): ${CACHE_SPEEDUP}x"
echo "  Cache speedup (nocache -> warm): ${NOCACHE_SPEEDUP}x"

# Get cache stats
export RNINJA_CACHE_ENABLED=1
CACHE_STATS=$($RNINJA -t cache-stats 2>/dev/null | grep -E "(hits|misses|size)" || echo "N/A")
echo ""
echo "Cache stats:"
echo "$CACHE_STATS"

# Write JSON results
cat > "$OUTPUT_FILE" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "size": "$SIZE",
    "targets": $TARGET_COUNT,
    "iterations": $ITERATIONS,
    "cold_cache_avg": $COLD_AVG,
    "warm_cache_avg": $WARM_AVG,
    "no_cache_avg": $NOCACHE_AVG,
    "cache_speedup": $CACHE_SPEEDUP,
    "nocache_speedup": $NOCACHE_SPEEDUP
}
EOF

echo ""
echo -e "${GREEN}Results written to $OUTPUT_FILE${NC}"

# Cleanup
cd - > /dev/null
rm -rf "$PROJECT_DIR" "$CACHE_DIR"
