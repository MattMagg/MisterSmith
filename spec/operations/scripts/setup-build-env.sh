#!/bin/bash
# setup-build-env.sh - Comprehensive build environment setup for MS Framework
# This script ensures all build dependencies and tools are properly installed

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Detect OS
OS_TYPE=$(uname -s)
ARCH_TYPE=$(uname -m)

log_info "Setting up MS Framework build environment..."
log_info "Detected OS: $OS_TYPE, Architecture: $ARCH_TYPE"

# Check for required commands
check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_error "$1 is not installed"
        return 1
    fi
    return 0
}

# Install Rust if not present
install_rust() {
    if ! check_command "rustc"; then
        log_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
        source "$HOME/.cargo/env"
    else
        log_info "Rust is already installed: $(rustc --version)"
    fi
}

# Install required Rust toolchain and components
setup_rust_toolchain() {
    log_info "Setting up Rust toolchain..."
    
    # Install specific Rust version
    rustup toolchain install 1.75.0
    rustup default 1.75.0
    
    # Add required components
    rustup component add rustfmt
    rustup component add clippy
    rustup component add rust-src
    rustup component add rust-analyzer
    rustup component add llvm-tools-preview
    
    # Add cross-compilation targets
    log_info "Adding cross-compilation targets..."
    rustup target add x86_64-unknown-linux-gnu
    rustup target add x86_64-unknown-linux-musl
    rustup target add aarch64-unknown-linux-gnu
    rustup target add aarch64-unknown-linux-musl
    rustup target add x86_64-apple-darwin
    rustup target add aarch64-apple-darwin
    rustup target add x86_64-pc-windows-gnu
    rustup target add x86_64-pc-windows-msvc
    rustup target add wasm32-unknown-unknown
    rustup target add wasm32-wasi
}

# Install cargo extensions
install_cargo_tools() {
    log_info "Installing cargo build tools..."
    
    # Essential build tools
    cargo install --locked cargo-watch || log_warn "cargo-watch installation failed"
    cargo install --locked cargo-nextest || log_warn "cargo-nextest installation failed"
    cargo install --locked cargo-audit || log_warn "cargo-audit installation failed"
    cargo install --locked cargo-deny || log_warn "cargo-deny installation failed"
    cargo install --locked cargo-outdated || log_warn "cargo-outdated installation failed"
    cargo install --locked cargo-machete || log_warn "cargo-machete installation failed"
    
    # Cross-compilation tool
    cargo install --locked cross || log_warn "cross installation failed"
    
    # Performance analysis tools
    cargo install --locked cargo-bloat || log_warn "cargo-bloat installation failed"
    cargo install --locked cargo-llvm-lines || log_warn "cargo-llvm-lines installation failed"
    cargo install --locked cargo-expand || log_warn "cargo-expand installation failed"
    
    # Documentation tools
    cargo install --locked cargo-docs-rs || log_warn "cargo-docs-rs installation failed"
    
    # Benchmarking tools
    cargo install --locked cargo-criterion || log_warn "cargo-criterion installation failed"
    
    # Security tools
    cargo install --locked cargo-geiger || log_warn "cargo-geiger installation failed"
}

# Install system dependencies based on OS
install_system_deps() {
    case "$OS_TYPE" in
        Linux*)
            log_info "Installing Linux system dependencies..."
            if command -v apt-get &> /dev/null; then
                sudo apt-get update
                sudo apt-get install -y \
                    build-essential \
                    pkg-config \
                    libssl-dev \
                    cmake \
                    clang \
                    lld \
                    libudev-dev \
                    libseccomp-dev \
                    libsystemd-dev \
                    protobuf-compiler
            elif command -v yum &> /dev/null; then
                sudo yum install -y \
                    gcc \
                    gcc-c++ \
                    make \
                    pkgconfig \
                    openssl-devel \
                    cmake \
                    clang \
                    lld \
                    systemd-devel \
                    protobuf-compiler
            fi
            
            # Install mold linker for faster builds
            if ! check_command "mold"; then
                log_info "Installing mold linker..."
                curl -LO https://github.com/rui314/mold/releases/download/v2.3.3/mold-2.3.3-x86_64-linux.tar.gz
                tar xzf mold-2.3.3-x86_64-linux.tar.gz
                sudo cp -r mold-2.3.3-x86_64-linux/* /usr/local/
                rm -rf mold-2.3.3-x86_64-linux*
            fi
            ;;
            
        Darwin*)
            log_info "Installing macOS system dependencies..."
            if command -v brew &> /dev/null; then
                brew update
                brew install \
                    pkg-config \
                    openssl \
                    cmake \
                    protobuf \
                    llvm
            else
                log_error "Homebrew not found. Please install Homebrew first."
                exit 1
            fi
            ;;
            
        MINGW*|MSYS*|CYGWIN*)
            log_info "Windows detected. Ensuring build tools..."
            log_warn "Please ensure Visual Studio Build Tools 2022 or later is installed"
            log_warn "Download from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022"
            ;;
            
        *)
            log_warn "Unknown OS: $OS_TYPE. You may need to install dependencies manually."
            ;;
    esac
}

# Setup sccache for faster builds
setup_sccache() {
    log_info "Setting up sccache..."
    if ! check_command "sccache"; then
        cargo install --locked sccache
    fi
    
    # Configure sccache
    export RUSTC_WRAPPER="sccache"
    export SCCACHE_CACHE_SIZE="10G"
    
    # Add to shell profile
    echo 'export RUSTC_WRAPPER="sccache"' >> ~/.bashrc
    echo 'export SCCACHE_CACHE_SIZE="10G"' >> ~/.bashrc
}

# Create development configuration
setup_dev_config() {
    log_info "Setting up development configuration..."
    
    # Create .cargo/config.toml if it doesn't exist
    mkdir -p .cargo
    
    cat > .cargo/config.toml << 'EOF'
[build]
# Use all available CPU cores
jobs = -1
# Enable incremental compilation
incremental = true
# Use sccache if available
rustc-wrapper = "sccache"

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = [
    "-C", "link-arg=-fuse-ld=mold",
    "-C", "target-cpu=x86-64-v2",
    "-Z", "share-generics=y"
]

[target.x86_64-apple-darwin]
rustflags = [
    "-C", "target-cpu=native",
    "-C", "link-arg=-undefined",
    "-C", "link-arg=dynamic_lookup",
]

[target.aarch64-apple-darwin]
rustflags = [
    "-C", "target-cpu=apple-m1",
]

[alias]
# Development shortcuts
dev = "build --features dev"
prod = "build --release --features tier_3"
test-all = "nextest run --all-features"
bench-all = "bench --all-features"
doc-all = "doc --all-features --no-deps --open"
audit-deps = "audit --deny warnings"
check-all = "check --all-targets --all-features"
fix = "clippy --all-targets --all-features --fix --allow-dirty --allow-staged"

# Cross-compilation shortcuts
build-linux = "build --target x86_64-unknown-linux-gnu --features tier_3"
build-linux-musl = "build --target x86_64-unknown-linux-musl --features tier_3"
build-macos = "build --target x86_64-apple-darwin --features tier_3"
build-windows = "build --target x86_64-pc-windows-gnu --features tier_3"
build-wasm = "build --target wasm32-unknown-unknown --features wasm"
EOF
}

# Setup git hooks
setup_git_hooks() {
    log_info "Setting up git hooks..."
    
    if [ -d .git ]; then
        mkdir -p .git/hooks
        
        # Pre-commit hook
        cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Format check
echo "Checking formatting..."
cargo fmt -- --check

# Clippy check
echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Test check
echo "Running tests..."
cargo nextest run --all-features

echo "Pre-commit checks passed!"
EOF
        
        chmod +x .git/hooks/pre-commit
        
        # Pre-push hook
        cat > .git/hooks/pre-push << 'EOF'
#!/bin/bash
set -e

echo "Running pre-push checks..."

# Security audit
echo "Running security audit..."
cargo audit

# Check for outdated dependencies
echo "Checking for outdated dependencies..."
cargo outdated

echo "Pre-push checks passed!"
EOF
        
        chmod +x .git/hooks/pre-push
    fi
}

# Verify installation
verify_setup() {
    log_info "Verifying build environment setup..."
    
    local all_good=true
    
    # Check Rust installation
    if rustc --version &> /dev/null; then
        log_info "✓ Rust compiler: $(rustc --version)"
    else
        log_error "✗ Rust compiler not found"
        all_good=false
    fi
    
    # Check cargo
    if cargo --version &> /dev/null; then
        log_info "✓ Cargo: $(cargo --version)"
    else
        log_error "✗ Cargo not found"
        all_good=false
    fi
    
    # Check important cargo tools
    for tool in cargo-nextest cargo-audit cross; do
        if cargo $tool --version &> /dev/null 2>&1; then
            log_info "✓ $tool installed"
        else
            log_warn "✗ $tool not installed (optional)"
        fi
    done
    
    # Check cross-compilation targets
    log_info "Checking cross-compilation targets..."
    rustup target list --installed
    
    if $all_good; then
        log_info "Build environment setup completed successfully!"
    else
        log_error "Build environment setup incomplete. Please check errors above."
        exit 1
    fi
}

# Main execution
main() {
    install_rust
    setup_rust_toolchain
    install_system_deps
    install_cargo_tools
    setup_sccache
    setup_dev_config
    setup_git_hooks
    verify_setup
    
    log_info "Build environment setup complete!"
    log_info "You can now build the MS Framework with: cargo build --features tier_3"
}

# Run main function
main "$@"