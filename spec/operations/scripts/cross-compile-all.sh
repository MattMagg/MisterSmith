#!/bin/bash
# cross-compile-all.sh - Cross-compile MS Framework for all supported platforms
# This script builds the framework for multiple architectures and operating systems

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
OUTPUT_DIR="${OUTPUT_DIR:-$PROJECT_ROOT/dist}"
FEATURES="${FEATURES:-tier_3}"
PARALLEL_JOBS="${PARALLEL_JOBS:-4}"

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
log_build() { echo -e "${BLUE}[BUILD]${NC} $1"; }

# Target configurations
declare -A TARGETS=(
    ["x86_64-unknown-linux-gnu"]="linux/amd64"
    ["x86_64-unknown-linux-musl"]="linux/amd64-musl"
    ["aarch64-unknown-linux-gnu"]="linux/arm64"
    ["aarch64-unknown-linux-musl"]="linux/arm64-musl"
    ["armv7-unknown-linux-gnueabihf"]="linux/armv7"
    ["x86_64-apple-darwin"]="darwin/amd64"
    ["aarch64-apple-darwin"]="darwin/arm64"
    ["x86_64-pc-windows-gnu"]="windows/amd64"
    ["x86_64-pc-windows-msvc"]="windows/amd64-msvc"
    ["wasm32-unknown-unknown"]="wasm/unknown"
    ["wasm32-wasi"]="wasm/wasi"
)

# Build configurations per target
declare -A BUILD_CONFIGS=(
    ["wasm32-unknown-unknown"]="--features wasm --no-default-features"
    ["wasm32-wasi"]="--features wasm --no-default-features"
)

# Check if cross is available
check_cross() {
    if command -v cross &> /dev/null; then
        echo "cross"
    else
        log_warn "cross not found, using cargo (some targets may fail)"
        echo "cargo"
    fi
}

# Build for a specific target
build_target() {
    local target=$1
    local platform=$2
    local build_tool=$3
    
    log_build "Building for $target ($platform)..."
    
    # Get build configuration for this target
    local build_config="${BUILD_CONFIGS[$target]:-"--features $FEATURES"}"
    
    # Create output directory
    local output_path="$OUTPUT_DIR/$platform"
    mkdir -p "$output_path"
    
    # Build command
    local build_cmd="$build_tool build --release --target $target $build_config"
    
    # Execute build
    if $build_cmd; then
        # Copy artifacts
        local binary_name="mister-smith-framework"
        local source_file="target/$target/release/$binary_name"
        
        # Handle Windows executables
        if [[ "$target" == *"windows"* ]]; then
            source_file="$source_file.exe"
            binary_name="$binary_name.exe"
        fi
        
        # Handle WASM files
        if [[ "$target" == "wasm32"* ]]; then
            # Copy all WASM files
            find "target/$target/release" -name "*.wasm" -exec cp {} "$output_path/" \;
            log_info "✓ Built WASM modules for $platform"
        elif [[ -f "$source_file" ]]; then
            cp "$source_file" "$output_path/$binary_name"
            
            # Strip binary for size optimization (except Windows and WASM)
            if [[ "$target" != *"windows"* ]] && [[ "$target" != "wasm32"* ]]; then
                strip "$output_path/$binary_name" 2>/dev/null || true
            fi
            
            # Create archive
            local archive_name="mister-smith-$target.tar.gz"
            if [[ "$target" == *"windows"* ]]; then
                archive_name="mister-smith-$target.zip"
            fi
            
            (
                cd "$output_path"
                if [[ "$target" == *"windows"* ]]; then
                    zip "$archive_name" "$binary_name"
                else
                    tar czf "$archive_name" "$binary_name"
                fi
            )
            
            log_info "✓ Built $target → $output_path/$archive_name"
        else
            log_error "✗ Build succeeded but binary not found for $target"
            return 1
        fi
    else
        log_error "✗ Build failed for $target"
        return 1
    fi
}

# Build targets in parallel
build_parallel() {
    local build_tool=$1
    local pids=()
    local failed_targets=()
    
    # Start builds in parallel
    for target in "${!TARGETS[@]}"; do
        while [[ $(jobs -r -p | wc -l) -ge $PARALLEL_JOBS ]]; do
            sleep 1
        done
        
        (
            build_target "$target" "${TARGETS[$target]}" "$build_tool"
        ) &
        
        pids+=($!)
    done
    
    # Wait for all builds to complete
    for pid in "${pids[@]}"; do
        if ! wait "$pid"; then
            failed_targets+=("failed")
        fi
    done
    
    return ${#failed_targets[@]}
}

# Build targets sequentially
build_sequential() {
    local build_tool=$1
    local failed_count=0
    
    for target in "${!TARGETS[@]}"; do
        if ! build_target "$target" "${TARGETS[$target]}" "$build_tool"; then
            ((failed_count++))
        fi
    done
    
    return $failed_count
}

# Generate build matrix report
generate_build_report() {
    local report_file="$OUTPUT_DIR/build-report.md"
    
    cat > "$report_file" << EOF
# MS Framework Cross-Compilation Report

Generated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")

## Build Configuration
- Features: $FEATURES
- Project Root: $PROJECT_ROOT
- Output Directory: $OUTPUT_DIR

## Build Matrix

| Target | Platform | Status | Size | Archive |
|--------|----------|--------|------|---------|
EOF
    
    for target in "${!TARGETS[@]}"; do
        local platform="${TARGETS[$target]}"
        local status="❌"
        local size="-"
        local archive="-"
        
        # Check if build succeeded
        if [[ -f "$OUTPUT_DIR/$platform/mister-smith-$target.tar.gz" ]] || \
           [[ -f "$OUTPUT_DIR/$platform/mister-smith-$target.zip" ]]; then
            status="✅"
            
            # Get file size
            local file
            if [[ -f "$OUTPUT_DIR/$platform/mister-smith-$target.tar.gz" ]]; then
                file="$OUTPUT_DIR/$platform/mister-smith-$target.tar.gz"
            else
                file="$OUTPUT_DIR/$platform/mister-smith-$target.zip"
            fi
            
            size=$(du -h "$file" | cut -f1)
            archive=$(basename "$file")
        fi
        
        echo "| $target | $platform | $status | $size | $archive |" >> "$report_file"
    done
    
    cat >> "$report_file" << EOF

## Checksums

\`\`\`
$(cd "$OUTPUT_DIR" && find . -name "*.tar.gz" -o -name "*.zip" | xargs sha256sum 2>/dev/null || echo "No archives found")
\`\`\`

## Build Logs

Check individual build logs in CI/CD system for detailed information.
EOF
    
    log_info "Build report generated: $report_file"
}

# Setup build environment
setup_environment() {
    log_info "Setting up build environment..."
    
    # Ensure Rust is installed
    if ! command -v rustc &> /dev/null; then
        log_error "Rust is not installed. Please install Rust first."
        exit 1
    fi
    
    # Add required targets
    log_info "Adding cross-compilation targets..."
    for target in "${!TARGETS[@]}"; do
        rustup target add "$target" 2>/dev/null || log_warn "Failed to add target: $target"
    done
    
    # Create output directory
    mkdir -p "$OUTPUT_DIR"
    
    # Clean previous builds
    if [[ -d "$OUTPUT_DIR" ]] && [[ "$CLEAN_BUILD" == "true" ]]; then
        log_info "Cleaning previous builds..."
        rm -rf "$OUTPUT_DIR"/*
    fi
}

# Main build process
main() {
    local start_time=$(date +%s)
    
    log_info "MS Framework Cross-Compilation Build"
    log_info "===================================="
    
    # Parse arguments
    local parallel_mode=true
    local clean_build=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --sequential)
                parallel_mode=false
                shift
                ;;
            --clean)
                clean_build=true
                CLEAN_BUILD=true
                shift
                ;;
            --features)
                FEATURES="$2"
                shift 2
                ;;
            --output)
                OUTPUT_DIR="$2"
                shift 2
                ;;
            --jobs)
                PARALLEL_JOBS="$2"
                shift 2
                ;;
            --help)
                cat << EOF
Usage: $0 [OPTIONS]

Options:
    --sequential    Build targets sequentially instead of in parallel
    --clean         Clean previous builds before starting
    --features      Cargo features to enable (default: tier_3)
    --output        Output directory (default: ./dist)
    --jobs          Number of parallel jobs (default: 4)
    --help          Show this help message

Examples:
    $0                          # Build all targets with default settings
    $0 --clean --features tier_2    # Clean build with tier_2 features
    $0 --sequential --jobs 1        # Sequential build
EOF
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Setup environment
    setup_environment
    
    # Determine build tool
    local build_tool=$(check_cross)
    log_info "Using build tool: $build_tool"
    
    # Run builds
    local failed_count
    if [[ "$parallel_mode" == "true" ]]; then
        log_info "Building targets in parallel (max $PARALLEL_JOBS jobs)..."
        build_parallel "$build_tool"
        failed_count=$?
    else
        log_info "Building targets sequentially..."
        build_sequential "$build_tool"
        failed_count=$?
    fi
    
    # Generate build report
    generate_build_report
    
    # Calculate build time
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    local minutes=$((duration / 60))
    local seconds=$((duration % 60))
    
    log_info "Build completed in ${minutes}m ${seconds}s"
    
    # Summary
    local total_targets=${#TARGETS[@]}
    local succeeded=$((total_targets - failed_count))
    
    echo
    log_info "Build Summary:"
    log_info "=============="
    log_info "Total targets: $total_targets"
    log_info "Succeeded: $succeeded"
    log_info "Failed: $failed_count"
    
    if [[ $failed_count -eq 0 ]]; then
        log_info "All builds completed successfully! 🎉"
    else
        log_warn "Some builds failed. Check logs for details."
        exit 1
    fi
}

# Run main
main "$@"