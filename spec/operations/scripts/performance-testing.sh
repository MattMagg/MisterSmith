#!/bin/bash
# performance-testing.sh - Comprehensive performance testing suite for MS Framework
# This script runs various performance tests including benchmarks, load tests, and profiling

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
RESULTS_DIR="${RESULTS_DIR:-$PROJECT_ROOT/perf-results}"
TEST_DURATION="${TEST_DURATION:-60}"
LOAD_LEVELS="${LOAD_LEVELS:-1,10,50,100,500}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging
log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_perf() { echo -e "${BLUE}[PERF]${NC} $1"; }

# Performance test types
declare -A TEST_TYPES=(
    ["micro"]="Micro-benchmarks for individual components"
    ["macro"]="End-to-end scenario benchmarks"
    ["load"]="Load testing with various concurrency levels"
    ["stress"]="Stress testing to find breaking points"
    ["memory"]="Memory usage and leak detection"
    ["cpu"]="CPU profiling and optimization"
)

# Check required tools
check_dependencies() {
    local missing=()
    
    # Required tools
    local tools=(
        "cargo"
        "hyperfine"
        "wrk"
        "curl"
        "jq"
    )
    
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            missing+=("$tool")
        fi
    done
    
    # Install missing tools if possible
    if [[ ${#missing[@]} -gt 0 ]]; then
        log_warn "Missing tools: ${missing[*]}"
        
        for tool in "${missing[@]}"; do
            case "$tool" in
                hyperfine)
                    cargo install hyperfine --locked || log_error "Failed to install hyperfine"
                    ;;
                wrk)
                    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
                        sudo apt-get update && sudo apt-get install -y wrk || log_error "Failed to install wrk"
                    elif [[ "$OSTYPE" == "darwin"* ]]; then
                        brew install wrk || log_error "Failed to install wrk"
                    fi
                    ;;
                *)
                    log_error "Please install $tool manually"
                    ;;
            esac
        done
    fi
}

# Install performance testing tools
install_perf_tools() {
    log_info "Installing performance testing tools..."
    
    # Rust performance tools
    cargo install --locked cargo-criterion || log_warn "cargo-criterion install failed"
    cargo install --locked cargo-bench || log_warn "cargo-bench install failed"
    cargo install --locked critcmp || log_warn "critcmp install failed"
    cargo install --locked cargo-flamegraph || log_warn "cargo-flamegraph install failed"
    cargo install --locked cargo-profdata || log_warn "cargo-profdata install failed"
    cargo install --locked cargo-llvm-lines || log_warn "cargo-llvm-lines install failed"
}

# Build optimized binary for testing
build_perf_binary() {
    log_info "Building optimized binary for performance testing..."
    
    # Build with maximum optimizations
    RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C lto=fat" \
        cargo build --release --features "tier_3 performance" \
        --profile maxperf
    
    # Verify binary exists
    local binary="$PROJECT_ROOT/target/release/mister-smith-framework"
    if [[ ! -f "$binary" ]]; then
        log_error "Performance binary not found: $binary"
        return 1
    fi
    
    log_info "Performance binary built: $binary"
    log_info "Binary size: $(du -h "$binary" | cut -f1)"
}

# Run micro-benchmarks
run_micro_benchmarks() {
    log_perf "Running micro-benchmarks..."
    
    local results_dir="$RESULTS_DIR/micro"
    mkdir -p "$results_dir"
    
    # Run Criterion benchmarks
    log_info "Running Criterion benchmarks..."
    cargo bench --features "tier_3 performance" -- --output-format json \
        > "$results_dir/criterion.json" 2>&1 || log_warn "Some benchmarks failed"
    
    # Generate benchmark report
    if command -v cargo-criterion &> /dev/null; then
        cargo criterion --message-format json \
            > "$results_dir/criterion-report.json" 2>&1 || true
    fi
    
    # Component-specific benchmarks
    log_info "Running component benchmarks..."
    
    # Agent scheduling benchmarks
    cargo bench --bench agent_scheduling \
        > "$results_dir/agent_scheduling.txt" 2>&1 || true
    
    # Message passing benchmarks
    cargo bench --bench message_passing \
        > "$results_dir/message_passing.txt" 2>&1 || true
    
    # Tool execution benchmarks
    cargo bench --bench tool_execution \
        > "$results_dir/tool_execution.txt" 2>&1 || true
    
    log_info "Micro-benchmarks completed. Results in: $results_dir"
}

# Run macro-benchmarks (end-to-end scenarios)
run_macro_benchmarks() {
    log_perf "Running macro-benchmarks..."
    
    local results_dir="$RESULTS_DIR/macro"
    mkdir -p "$results_dir"
    
    # Start framework instance for testing
    local binary="$PROJECT_ROOT/target/release/mister-smith-framework"
    local config_file="$PROJECT_ROOT/test-configs/performance.toml"
    
    # Create test configuration
    mkdir -p "$(dirname "$config_file")"
    cat > "$config_file" << EOF
[server]
host = "127.0.0.1"
port = 8080

[agents]
max_agents = 1000
spawn_timeout = "5s"

[monitoring]
metrics_enabled = true
metrics_port = 9090

[performance]
max_concurrent_tasks = 500
task_timeout = "30s"
EOF
    
    # Start framework
    log_info "Starting framework for macro-benchmarks..."
    "$binary" serve --config "$config_file" &
    local framework_pid=$!
    
    # Wait for startup
    local retries=30
    while [[ $retries -gt 0 ]]; do
        if curl -s http://127.0.0.1:8080/health &> /dev/null; then
            break
        fi
        sleep 1
        ((retries--))
    done
    
    if [[ $retries -eq 0 ]]; then
        log_error "Framework failed to start"
        kill $framework_pid 2>/dev/null || true
        return 1
    fi
    
    log_info "Framework started successfully"
    
    # Run scenario benchmarks
    
    # 1. Agent creation benchmark
    log_info "Benchmarking agent creation..."
    hyperfine --warmup 3 --runs 10 \
        --export-json "$results_dir/agent_creation.json" \
        "curl -s -X POST http://127.0.0.1:8080/agents -d '{\"type\":\"simple\"}'"
    
    # 2. Task execution benchmark
    log_info "Benchmarking task execution..."
    hyperfine --warmup 3 --runs 10 \
        --export-json "$results_dir/task_execution.json" \
        "curl -s -X POST http://127.0.0.1:8080/tasks -d '{\"action\":\"echo\",\"data\":\"hello\"}'"
    
    # 3. Message processing benchmark
    log_info "Benchmarking message processing..."
    hyperfine --warmup 3 --runs 10 \
        --export-json "$results_dir/message_processing.json" \
        "curl -s -X POST http://127.0.0.1:8080/messages -d '{\"type\":\"broadcast\",\"payload\":\"test\"}'"
    
    # Cleanup
    kill $framework_pid 2>/dev/null || true
    wait $framework_pid 2>/dev/null || true
    
    log_info "Macro-benchmarks completed. Results in: $results_dir"
}

# Run load tests
run_load_tests() {
    log_perf "Running load tests..."
    
    local results_dir="$RESULTS_DIR/load"
    mkdir -p "$results_dir"
    
    # Start framework for load testing
    local binary="$PROJECT_ROOT/target/release/mister-smith-framework"
    local config_file="$PROJECT_ROOT/test-configs/load-test.toml"
    
    # Create load test configuration
    cat > "$config_file" << EOF
[server]
host = "127.0.0.1"
port = 8080
max_connections = 10000

[agents]
max_agents = 5000
pool_size = 100

[performance]
max_concurrent_tasks = 2000
worker_threads = 8
EOF
    
    # Start framework
    "$binary" serve --config "$config_file" &
    local framework_pid=$!
    
    # Wait for startup
    sleep 5
    
    # Run load tests with different concurrency levels
    IFS=',' read -ra LEVELS <<< "$LOAD_LEVELS"
    
    for level in "${LEVELS[@]}"; do
        log_info "Running load test with $level concurrent connections..."
        
        # HTTP load test
        wrk -t4 -c"$level" -d"$TEST_DURATION"s --latency \
            http://127.0.0.1:8080/health \
            > "$results_dir/http_load_${level}.txt" 2>&1
        
        # API load test
        wrk -t4 -c"$level" -d"$TEST_DURATION"s --latency \
            -s "$SCRIPT_DIR/load-test-script.lua" \
            http://127.0.0.1:8080/ \
            > "$results_dir/api_load_${level}.txt" 2>&1
        
        # Give system time to recover
        sleep 10
    done
    
    # Cleanup
    kill $framework_pid 2>/dev/null || true
    wait $framework_pid 2>/dev/null || true
    
    log_info "Load tests completed. Results in: $results_dir"
}

# Create load test script for wrk
create_load_test_script() {
    cat > "$SCRIPT_DIR/load-test-script.lua" << 'EOF'
-- Load test script for wrk
local counter = 0

request = function()
    counter = counter + 1
    local method = "POST"
    local path = "/tasks"
    local headers = {
        ["Content-Type"] = "application/json"
    }
    local body = string.format('{"id": %d, "action": "test", "data": "load-test"}', counter)
    
    return wrk.format(method, path, headers, body)
end

response = function(status, headers, body)
    if status ~= 200 then
        wrk.thread:get("errors"):add(1)
    end
end

init = function(args)
    wrk.thread:set("errors", 0)
end

done = function(summary, latency, requests)
    local errors = wrk.thread:get("errors")
    io.write(string.format("Errors: %d\n", errors))
end
EOF
}

# Run stress tests
run_stress_tests() {
    log_perf "Running stress tests to find breaking points..."
    
    local results_dir="$RESULTS_DIR/stress"
    mkdir -p "$results_dir"
    
    # Start framework
    local binary="$PROJECT_ROOT/target/release/mister-smith-framework"
    "$binary" serve &
    local framework_pid=$!
    sleep 5
    
    # Gradually increase load until failure
    local connections=10
    local max_connections=10000
    local step_duration=30
    
    while [[ $connections -le $max_connections ]]; do
        log_info "Stress testing with $connections connections..."
        
        # Run stress test
        timeout $step_duration wrk -t8 -c"$connections" -d"$step_duration"s \
            http://127.0.0.1:8080/health \
            > "$results_dir/stress_${connections}.txt" 2>&1
        
        local exit_code=$?
        
        # Check if framework is still responding
        if ! curl -s http://127.0.0.1:8080/health &> /dev/null; then
            log_warn "Framework stopped responding at $connections connections"
            echo "$connections" > "$results_dir/breaking_point.txt"
            break
        fi
        
        # Check for high error rates
        local error_rate=$(grep -o "Socket errors: [0-9]*" "$results_dir/stress_${connections}.txt" | \
                          awk '{print $3}' || echo "0")
        
        if [[ $error_rate -gt $((connections / 2)) ]]; then
            log_warn "High error rate detected at $connections connections"
            echo "$connections" > "$results_dir/error_threshold.txt"
        fi
        
        # Increase load
        connections=$((connections * 2))
        sleep 5
    done
    
    # Cleanup
    kill $framework_pid 2>/dev/null || true
    
    log_info "Stress tests completed. Results in: $results_dir"
}

# Run memory profiling
run_memory_profiling() {
    log_perf "Running memory profiling..."
    
    local results_dir="$RESULTS_DIR/memory"
    mkdir -p "$results_dir"
    
    # Memory usage over time
    log_info "Monitoring memory usage over time..."
    
    local binary="$PROJECT_ROOT/target/release/mister-smith-framework"
    
    # Start framework with memory monitoring
    valgrind --tool=massif --massif-out-file="$results_dir/massif.out" \
        "$binary" serve &
    local valgrind_pid=$!
    
    # Generate load while monitoring
    sleep 10  # Let it start
    
    # Run moderate load for memory analysis
    wrk -t2 -c50 -d60s http://127.0.0.1:8080/health &
    local wrk_pid=$!
    
    # Monitor memory usage
    (
        echo "timestamp,memory_mb" > "$results_dir/memory_usage.csv"
        for i in {1..60}; do
            local memory=$(ps -o rss= -p $valgrind_pid 2>/dev/null | awk '{print $1/1024}' || echo "0")
            echo "$(date +%s),$memory" >> "$results_dir/memory_usage.csv"
            sleep 1
        done
    ) &
    local monitor_pid=$!
    
    # Wait for completion
    wait $wrk_pid 2>/dev/null || true
    sleep 5
    
    # Stop monitoring
    kill $valgrind_pid 2>/dev/null || true
    kill $monitor_pid 2>/dev/null || true
    
    # Generate memory report
    if command -v ms_print &> /dev/null; then
        ms_print "$results_dir/massif.out" > "$results_dir/memory_report.txt"
    fi
    
    log_info "Memory profiling completed. Results in: $results_dir"
}

# Run CPU profiling
run_cpu_profiling() {
    log_perf "Running CPU profiling..."
    
    local results_dir="$RESULTS_DIR/cpu"
    mkdir -p "$results_dir"
    
    # Flame graph generation
    if command -v cargo-flamegraph &> /dev/null; then
        log_info "Generating flame graph..."
        
        # Build with debug info for profiling
        cargo build --release --features "tier_3 profiling"
        
        # Generate flame graph
        cargo flamegraph --bin mister-smith-framework -- serve &
        local flamegraph_pid=$!
        
        # Generate load for profiling
        sleep 10
        wrk -t4 -c100 -d30s http://127.0.0.1:8080/health &
        wait $!
        
        # Stop profiling
        kill $flamegraph_pid 2>/dev/null || true
        
        # Move flame graph to results
        mv flamegraph.svg "$results_dir/" 2>/dev/null || true
    fi
    
    # perf profiling (Linux only)
    if command -v perf &> /dev/null && [[ "$OSTYPE" == "linux-gnu"* ]]; then
        log_info "Running perf profiling..."
        
        local binary="$PROJECT_ROOT/target/release/mister-smith-framework"
        
        # Start framework
        "$binary" serve &
        local framework_pid=$!
        sleep 5
        
        # Profile for 30 seconds under load
        perf record -g -p $framework_pid &
        local perf_pid=$!
        
        wrk -t2 -c50 -d30s http://127.0.0.1:8080/health
        
        kill $perf_pid 2>/dev/null || true
        kill $framework_pid 2>/dev/null || true
        
        # Generate perf report
        perf report > "$results_dir/perf_report.txt" 2>/dev/null || true
        mv perf.data "$results_dir/" 2>/dev/null || true
    fi
    
    log_info "CPU profiling completed. Results in: $results_dir"
}

# Generate performance report
generate_performance_report() {
    log_info "Generating performance report..."
    
    local report_file="$RESULTS_DIR/performance_report.md"
    
    cat > "$report_file" << EOF
# MS Framework Performance Report

Generated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")

## Test Configuration
- Test Duration: ${TEST_DURATION}s
- Load Levels: $LOAD_LEVELS
- Platform: $(uname -s) $(uname -m)
- Rust Version: $(rustc --version)

## Summary

EOF
    
    # Add micro-benchmark summary
    if [[ -f "$RESULTS_DIR/micro/criterion.json" ]]; then
        echo "### Micro-Benchmarks" >> "$report_file"
        echo "" >> "$report_file"
        # Parse criterion results and add to report
        jq -r '.[] | "- \(.benchmark_name): \(.mean) ± \(.std_dev)"' \
            "$RESULTS_DIR/micro/criterion.json" >> "$report_file" 2>/dev/null || true
        echo "" >> "$report_file"
    fi
    
    # Add load test summary
    if [[ -d "$RESULTS_DIR/load" ]]; then
        echo "### Load Test Results" >> "$report_file"
        echo "" >> "$report_file"
        echo "| Connections | Requests/sec | Avg Latency | 99th Percentile |" >> "$report_file"
        echo "|-------------|--------------|-------------|-----------------|" >> "$report_file"
        
        for file in "$RESULTS_DIR/load"/http_load_*.txt; do
            if [[ -f "$file" ]]; then
                local connections=$(basename "$file" .txt | sed 's/http_load_//')
                local rps=$(grep "Requests/sec:" "$file" | awk '{print $2}' || echo "N/A")
                local avg_lat=$(grep "Latency" "$file" | awk '{print $2}' || echo "N/A")
                local p99_lat=$(grep "99%" "$file" | awk '{print $2}' || echo "N/A")
                
                echo "| $connections | $rps | $avg_lat | $p99_lat |" >> "$report_file"
            fi
        done
        echo "" >> "$report_file"
    fi
    
    # Add stress test results
    if [[ -f "$RESULTS_DIR/stress/breaking_point.txt" ]]; then
        echo "### Stress Test Results" >> "$report_file"
        echo "" >> "$report_file"
        echo "- Breaking Point: $(cat "$RESULTS_DIR/stress/breaking_point.txt") connections" >> "$report_file"
        echo "" >> "$report_file"
    fi
    
    # Add memory analysis
    if [[ -f "$RESULTS_DIR/memory/memory_usage.csv" ]]; then
        echo "### Memory Usage" >> "$report_file"
        echo "" >> "$report_file"
        local max_memory=$(awk -F',' 'NR>1 {if($2>max) max=$2} END {print max}' "$RESULTS_DIR/memory/memory_usage.csv")
        echo "- Peak Memory Usage: ${max_memory}MB" >> "$report_file"
        echo "" >> "$report_file"
    fi
    
    # Add recommendations
    cat >> "$report_file" << EOF
## Recommendations

Based on the performance test results:

1. **Optimal Configuration**: 
   - Recommended max connections: [Analysis needed]
   - Suggested worker thread count: [Analysis needed]

2. **Performance Tuning**:
   - [Add specific recommendations based on results]

3. **Monitoring**:
   - Set up alerts for memory usage > [threshold]MB
   - Monitor request latency p99 < [threshold]ms

## Files

- Detailed results: \`$RESULTS_DIR\`
- Raw benchmark data: \`$RESULTS_DIR/micro/\`
- Load test outputs: \`$RESULTS_DIR/load/\`
- Profiling data: \`$RESULTS_DIR/cpu/\` and \`$RESULTS_DIR/memory/\`
EOF
    
    log_info "Performance report generated: $report_file"
}

# Main execution
main() {
    log_info "MS Framework Performance Testing Suite"
    log_info "======================================"
    
    # Parse arguments
    local test_types=()
    local all_tests=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --all)
                all_tests=true
                shift
                ;;
            --micro)
                test_types+=("micro")
                shift
                ;;
            --macro)
                test_types+=("macro")
                shift
                ;;
            --load)
                test_types+=("load")
                shift
                ;;
            --stress)
                test_types+=("stress")
                shift
                ;;
            --memory)
                test_types+=("memory")
                shift
                ;;
            --cpu)
                test_types+=("cpu")
                shift
                ;;
            --duration)
                TEST_DURATION="$2"
                shift 2
                ;;
            --load-levels)
                LOAD_LEVELS="$2"
                shift 2
                ;;
            --results-dir)
                RESULTS_DIR="$2"
                shift 2
                ;;
            --help)
                cat << EOF
Usage: $0 [OPTIONS]

Options:
    --all             Run all performance tests
    --micro           Run micro-benchmarks
    --macro           Run macro-benchmarks
    --load            Run load tests
    --stress          Run stress tests
    --memory          Run memory profiling
    --cpu             Run CPU profiling
    --duration SEC    Test duration in seconds (default: 60)
    --load-levels L   Comma-separated load levels (default: 1,10,50,100,500)
    --results-dir DIR Results directory (default: ./perf-results)
    --help            Show this help

Examples:
    $0 --all                    # Run all tests
    $0 --micro --load          # Run specific test types
    $0 --load --duration 120   # Extended load test
EOF
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Default to all tests if none specified
    if [[ $all_tests == true ]] || [[ ${#test_types[@]} -eq 0 ]]; then
        test_types=("micro" "macro" "load" "stress" "memory" "cpu")
    fi
    
    # Setup
    cd "$PROJECT_ROOT"
    mkdir -p "$RESULTS_DIR"
    
    # Check dependencies
    check_dependencies
    install_perf_tools
    
    # Create load test script
    create_load_test_script
    
    # Build performance binary
    build_perf_binary
    
    # Run selected tests
    local start_time=$(date +%s)
    
    for test_type in "${test_types[@]}"; do
        case "$test_type" in
            micro) run_micro_benchmarks ;;
            macro) run_macro_benchmarks ;;
            load) run_load_tests ;;
            stress) run_stress_tests ;;
            memory) run_memory_profiling ;;
            cpu) run_cpu_profiling ;;
            *) log_warn "Unknown test type: $test_type" ;;
        esac
    done
    
    # Generate report
    generate_performance_report
    
    # Summary
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log_info "Performance testing completed in ${duration}s"
    log_info "Results available in: $RESULTS_DIR"
    log_info "Performance report: $RESULTS_DIR/performance_report.md"
}

# Run main function
main "$@"