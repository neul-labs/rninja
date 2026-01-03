#!/bin/bash
# Remote cache benchmark
# Tests remote cache latency and throughput

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

RNINJA="$PROJECT_ROOT/target/release/rninja"
RNINJA_CACHED="$PROJECT_ROOT/target/release/rninja-cached"
SIZE="${1:-small}"
OUTPUT_FILE="$SCRIPT_DIR/remote_cache_results.json"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}=== rninja Remote Cache Benchmark ===${NC}"
echo ""

# Build binaries if needed
if [[ ! -x "$RNINJA" ]] || [[ ! -x "$RNINJA_CACHED" ]]; then
    echo "Building rninja..."
    cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml"
fi

# Check if rninja-cached exists
if [[ ! -x "$RNINJA_CACHED" ]]; then
    echo -e "${RED}Error: rninja-cached not found${NC}"
    echo "Remote cache server binary is required for this benchmark"
    exit 1
fi

# Set up directories
SERVER_DIR="/tmp/rninja-remote-cache-server"
PROJECT_DIR="/tmp/rninja-remote-bench"
SOCKET_PATH="/tmp/rninja-remote-bench.sock"

rm -rf "$SERVER_DIR" "$PROJECT_DIR"
mkdir -p "$SERVER_DIR"

# Generate project
"$SCRIPT_DIR/generate_project.sh" "$SIZE" "$PROJECT_DIR" > /dev/null

TARGET_COUNT=$(grep -c "^build " "$PROJECT_DIR/build.ninja")
echo "Project size: $SIZE ($TARGET_COUNT targets)"
echo ""

# Start cache server
echo -e "${YELLOW}Starting remote cache server...${NC}"
$RNINJA_CACHED --cache-dir "$SERVER_DIR" --socket "$SOCKET_PATH" &
SERVER_PID=$!

# Wait for server to start
sleep 1

if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo -e "${RED}Failed to start cache server${NC}"
    exit 1
fi

echo "Cache server running (PID: $SERVER_PID)"
echo ""

# Clean up function
cleanup() {
    echo "Cleaning up..."
    kill $SERVER_PID 2>/dev/null || true
    rm -rf "$SERVER_DIR" "$PROJECT_DIR" "$SOCKET_PATH"
}
trap cleanup EXIT

# Configure rninja to use remote cache
export RNINJA_CACHE_MODE=remote
export RNINJA_CACHE_SOCKET="$SOCKET_PATH"

cd "$PROJECT_DIR"

# Test 1: Cold remote cache (push)
echo -e "${YELLOW}=== Test 1: Cold Remote Cache (Push) ===${NC}"
rm -rf obj/*.o app 2>/dev/null || true
mkdir -p obj

start_time=$(date +%s.%N)
$RNINJA --no-daemon 2>/dev/null || true
end_time=$(date +%s.%N)

COLD_TIME=$(echo "$end_time - $start_time" | bc)
echo "  Time: ${COLD_TIME}s"

# Test 2: Warm remote cache (pull)
echo ""
echo -e "${YELLOW}=== Test 2: Warm Remote Cache (Pull) ===${NC}"
rm -rf obj/*.o app 2>/dev/null || true
mkdir -p obj

start_time=$(date +%s.%N)
$RNINJA --no-daemon 2>/dev/null || true
end_time=$(date +%s.%N)

WARM_TIME=$(echo "$end_time - $start_time" | bc)
echo "  Time: ${WARM_TIME}s"

# Test 3: Concurrent clients
echo ""
echo -e "${YELLOW}=== Test 3: Concurrent Clients ===${NC}"

# Create multiple project copies
for i in 1 2 3; do
    cp -r "$PROJECT_DIR" "/tmp/rninja-remote-bench-$i"
    rm -rf "/tmp/rninja-remote-bench-$i/obj/"*.o "/tmp/rninja-remote-bench-$i/app" 2>/dev/null || true
    mkdir -p "/tmp/rninja-remote-bench-$i/obj"
done

start_time=$(date +%s.%N)

# Run 3 builds concurrently
(cd /tmp/rninja-remote-bench-1 && $RNINJA --no-daemon 2>/dev/null) &
PID1=$!
(cd /tmp/rninja-remote-bench-2 && $RNINJA --no-daemon 2>/dev/null) &
PID2=$!
(cd /tmp/rninja-remote-bench-3 && $RNINJA --no-daemon 2>/dev/null) &
PID3=$!

wait $PID1 $PID2 $PID3

end_time=$(date +%s.%N)

CONCURRENT_TIME=$(echo "$end_time - $start_time" | bc)
echo "  3 concurrent builds: ${CONCURRENT_TIME}s"

# Cleanup concurrent test dirs
rm -rf /tmp/rninja-remote-bench-1 /tmp/rninja-remote-bench-2 /tmp/rninja-remote-bench-3

# Calculate speedup
if (( $(echo "$WARM_TIME > 0" | bc -l) )); then
    CACHE_SPEEDUP=$(echo "scale=2; $COLD_TIME / $WARM_TIME" | bc)
else
    CACHE_SPEEDUP="N/A"
fi

echo ""
echo -e "${GREEN}=== Remote Cache Benchmark Results ===${NC}"
echo "  Cold cache (push): ${COLD_TIME}s"
echo "  Warm cache (pull): ${WARM_TIME}s"
echo "  Concurrent (3x):   ${CONCURRENT_TIME}s"
echo "  Cache speedup:     ${CACHE_SPEEDUP}x"

# Write JSON results
cat > "$OUTPUT_FILE" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "size": "$SIZE",
    "targets": $TARGET_COUNT,
    "cold_cache_time": $COLD_TIME,
    "warm_cache_time": $WARM_TIME,
    "concurrent_time": $CONCURRENT_TIME,
    "cache_speedup": $CACHE_SPEEDUP
}
EOF

echo ""
echo -e "${GREEN}Results written to $OUTPUT_FILE${NC}"

cd - > /dev/null
