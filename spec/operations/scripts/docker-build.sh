#!/bin/bash
# docker-build.sh - Build MS Framework Docker images for multiple architectures
# Supports multi-arch builds, layer caching, and optimized image sizes

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
DOCKER_REGISTRY="${DOCKER_REGISTRY:-ghcr.io}"
DOCKER_NAMESPACE="${DOCKER_NAMESPACE:-mister-smith}"
IMAGE_NAME="${IMAGE_NAME:-ms-framework}"
VERSION="${VERSION:-latest}"

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
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

# Image variants
declare -A VARIANTS=(
    ["minimal"]="Minimal runtime image with core features only"
    ["standard"]="Standard image with tier_2 features"
    ["full"]="Full-featured image with all tier_3 features"
    ["debug"]="Debug image with development tools"
)

# Supported platforms
PLATFORMS="${PLATFORMS:-linux/amd64,linux/arm64}"

# Check Docker buildx availability
check_buildx() {
    if ! docker buildx version &> /dev/null; then
        log_error "Docker buildx is required for multi-platform builds"
        log_info "Install buildx: https://github.com/docker/buildx#installing"
        exit 1
    fi
}

# Setup buildx builder
setup_builder() {
    local builder_name="ms-framework-builder"
    
    # Check if builder exists
    if ! docker buildx ls | grep -q "$builder_name"; then
        log_info "Creating buildx builder: $builder_name"
        docker buildx create --name "$builder_name" --use --platform "$PLATFORMS"
        docker buildx inspect --bootstrap
    else
        log_info "Using existing builder: $builder_name"
        docker buildx use "$builder_name"
    fi
}

# Create Dockerfile for variant
create_dockerfile() {
    local variant=$1
    local dockerfile="Dockerfile.$variant"
    
    log_info "Creating $dockerfile..."
    
    case "$variant" in
        minimal)
            cat > "$PROJECT_ROOT/$dockerfile" << 'EOF'
# Build stage
FROM rust:1.75 AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY build.rs ./

# Build with minimal features
RUN cargo build --release --no-default-features --features "runtime actors tools"

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash msframework

# Copy binary from builder
COPY --from=builder /app/target/release/mister-smith-framework /usr/local/bin/

# Switch to non-root user
USER msframework
WORKDIR /home/msframework

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/mister-smith-framework", "health"]

# Default command
ENTRYPOINT ["/usr/local/bin/mister-smith-framework"]
CMD ["--help"]
EOF
            ;;
            
        standard)
            cat > "$PROJECT_ROOT/$dockerfile" << 'EOF'
# Build stage
FROM rust:1.75 AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY build.rs ./
COPY config-templates ./config-templates

# Build with tier_2 features
RUN cargo build --release --features tier_2

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash msframework

# Create required directories
RUN mkdir -p /etc/ms-framework /var/lib/ms-framework /var/log/ms-framework && \
    chown -R msframework:msframework /etc/ms-framework /var/lib/ms-framework /var/log/ms-framework

# Copy binary and configs from builder
COPY --from=builder /app/target/release/mister-smith-framework /usr/local/bin/
COPY --from=builder /app/config-templates /etc/ms-framework/templates

# Switch to non-root user
USER msframework
WORKDIR /home/msframework

# Environment variables
ENV MS_FRAMEWORK_CONFIG=/etc/ms-framework/config.toml
ENV MS_FRAMEWORK_DATA=/var/lib/ms-framework
ENV MS_FRAMEWORK_LOGS=/var/log/ms-framework

# Expose ports
EXPOSE 8080 9090

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
ENTRYPOINT ["/usr/local/bin/mister-smith-framework"]
CMD ["serve"]
EOF
            ;;
            
        full)
            cat > "$PROJECT_ROOT/$dockerfile" << 'EOF'
# Build stage
FROM rust:1.75 AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    clang \
    libpq-dev \
    libsqlite3-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy everything
COPY . .

# Build with all features
RUN cargo build --release --features tier_3

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    libsqlite3-0 \
    curl \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash msframework

# Create required directories
RUN mkdir -p /etc/ms-framework /var/lib/ms-framework /var/log/ms-framework /opt/ms-framework/plugins && \
    chown -R msframework:msframework /etc/ms-framework /var/lib/ms-framework /var/log/ms-framework /opt/ms-framework

# Copy binary and assets from builder
COPY --from=builder /app/target/release/mister-smith-framework /usr/local/bin/
COPY --from=builder /app/config-templates /etc/ms-framework/templates
COPY --from=builder /app/scripts /opt/ms-framework/scripts

# Switch to non-root user
USER msframework
WORKDIR /home/msframework

# Environment variables
ENV MS_FRAMEWORK_CONFIG=/etc/ms-framework/config.toml
ENV MS_FRAMEWORK_DATA=/var/lib/ms-framework
ENV MS_FRAMEWORK_LOGS=/var/log/ms-framework
ENV MS_FRAMEWORK_PLUGINS=/opt/ms-framework/plugins
ENV RUST_LOG=info

# Expose ports
EXPOSE 8080 9090 50051

# Volume mounts
VOLUME ["/etc/ms-framework", "/var/lib/ms-framework", "/var/log/ms-framework"]

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
ENTRYPOINT ["/usr/local/bin/mister-smith-framework"]
CMD ["serve", "--config", "/etc/ms-framework/config.toml"]
EOF
            ;;
            
        debug)
            cat > "$PROJECT_ROOT/$dockerfile" << 'EOF'
# Debug image with development tools
FROM rust:1.75

# Install development and debugging tools
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    clang \
    libpq-dev \
    libsqlite3-dev \
    protobuf-compiler \
    gdb \
    valgrind \
    strace \
    htop \
    vim \
    curl \
    jq \
    net-tools \
    tcpdump \
    && rm -rf /var/lib/apt/lists/*

# Install Rust debugging tools
RUN rustup component add rust-src rust-analyzer

WORKDIR /app

# Copy everything
COPY . .

# Build with debug symbols
RUN cargo build --features "tier_3 dev" --profile release-with-debug

# Create directories
RUN mkdir -p /etc/ms-framework /var/lib/ms-framework /var/log/ms-framework

# Environment variables
ENV MS_FRAMEWORK_CONFIG=/etc/ms-framework/config.toml
ENV MS_FRAMEWORK_DATA=/var/lib/ms-framework
ENV MS_FRAMEWORK_LOGS=/var/log/ms-framework
ENV RUST_LOG=debug
ENV RUST_BACKTRACE=full

# Keep source code for debugging
WORKDIR /app

# Expose all ports
EXPOSE 8080 9090 50051 6831 9411

# Volume mounts
VOLUME ["/etc/ms-framework", "/var/lib/ms-framework", "/var/log/ms-framework", "/app"]

# Default to bash for debugging
CMD ["/bin/bash"]
EOF
            ;;
    esac
}

# Build Docker image
build_image() {
    local variant=$1
    local tag_base="$DOCKER_REGISTRY/$DOCKER_NAMESPACE/$IMAGE_NAME"
    local dockerfile="Dockerfile.$variant"
    
    log_step "Building $variant image..."
    
    # Create Dockerfile
    create_dockerfile "$variant"
    
    # Build tags
    local tags=(
        "$tag_base:$VERSION-$variant"
        "$tag_base:$variant"
    )
    
    # Add latest tag for standard variant
    if [[ "$variant" == "standard" ]]; then
        tags+=("$tag_base:$VERSION")
        if [[ "$VERSION" == "latest" ]]; then
            tags+=("$tag_base:latest")
        fi
    fi
    
    # Build command
    local build_cmd="docker buildx build"
    build_cmd="$build_cmd --platform $PLATFORMS"
    build_cmd="$build_cmd --file $dockerfile"
    
    # Add tags
    for tag in "${tags[@]}"; do
        build_cmd="$build_cmd --tag $tag"
    done
    
    # Add build args
    build_cmd="$build_cmd --build-arg VERSION=$VERSION"
    build_cmd="$build_cmd --build-arg BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ')"
    build_cmd="$build_cmd --build-arg VCS_REF=$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
    
    # Cache configuration
    build_cmd="$build_cmd --cache-from type=registry,ref=$tag_base:buildcache-$variant"
    build_cmd="$build_cmd --cache-to type=registry,ref=$tag_base:buildcache-$variant,mode=max"
    
    # Push if not dry run
    if [[ "${DRY_RUN:-false}" != "true" ]]; then
        build_cmd="$build_cmd --push"
    fi
    
    # Add context
    build_cmd="$build_cmd $PROJECT_ROOT"
    
    # Execute build
    log_info "Executing: $build_cmd"
    if $build_cmd; then
        log_info "✓ Successfully built $variant image"
        
        # Show image info
        for tag in "${tags[@]}"; do
            log_info "  Tagged: $tag"
        done
    else
        log_error "✗ Failed to build $variant image"
        return 1
    fi
}

# Build all variants
build_all_variants() {
    local failed=0
    
    for variant in "${!VARIANTS[@]}"; do
        if [[ -n "${BUILD_VARIANT:-}" ]] && [[ "$BUILD_VARIANT" != "$variant" ]]; then
            continue
        fi
        
        if ! build_image "$variant"; then
            ((failed++))
        fi
    done
    
    return $failed
}

# Generate compose file
generate_compose_file() {
    log_info "Generating docker-compose.yml..."
    
    cat > "$PROJECT_ROOT/docker-compose.yml" << EOF
version: '3.8'

services:
  ms-framework:
    image: $DOCKER_REGISTRY/$DOCKER_NAMESPACE/$IMAGE_NAME:$VERSION
    container_name: ms-framework
    restart: unless-stopped
    ports:
      - "8080:8080"
      - "9090:9090"
      - "50051:50051"
    environment:
      - RUST_LOG=info
      - MS_FRAMEWORK_CONFIG=/etc/ms-framework/config.toml
    volumes:
      - ./config:/etc/ms-framework
      - ms-framework-data:/var/lib/ms-framework
      - ms-framework-logs:/var/log/ms-framework
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 5s
      retries: 3
    networks:
      - ms-framework-net

  postgres:
    image: postgres:16-alpine
    container_name: ms-framework-postgres
    restart: unless-stopped
    environment:
      - POSTGRES_DB=ms_framework
      - POSTGRES_USER=ms_framework
      - POSTGRES_PASSWORD=\${POSTGRES_PASSWORD:-changeme}
    volumes:
      - postgres-data:/var/lib/postgresql/data
    networks:
      - ms-framework-net

  redis:
    image: redis:7-alpine
    container_name: ms-framework-redis
    restart: unless-stopped
    command: redis-server --appendonly yes
    volumes:
      - redis-data:/data
    networks:
      - ms-framework-net

  prometheus:
    image: prom/prometheus:latest
    container_name: ms-framework-prometheus
    restart: unless-stopped
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    ports:
      - "9091:9090"
    networks:
      - ms-framework-net

volumes:
  ms-framework-data:
  ms-framework-logs:
  postgres-data:
  redis-data:
  prometheus-data:

networks:
  ms-framework-net:
    driver: bridge
EOF
    
    # Generate example prometheus config
    cat > "$PROJECT_ROOT/prometheus.yml" << EOF
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'ms-framework'
    static_configs:
      - targets: ['ms-framework:9090']
EOF
}

# Login to registry
login_registry() {
    if [[ -n "${DOCKER_USERNAME:-}" ]] && [[ -n "${DOCKER_PASSWORD:-}" ]]; then
        log_info "Logging in to $DOCKER_REGISTRY..."
        echo "$DOCKER_PASSWORD" | docker login "$DOCKER_REGISTRY" -u "$DOCKER_USERNAME" --password-stdin
    elif [[ -n "${GITHUB_TOKEN:-}" ]] && [[ "$DOCKER_REGISTRY" == "ghcr.io" ]]; then
        log_info "Logging in to GitHub Container Registry..."
        echo "$GITHUB_TOKEN" | docker login ghcr.io -u "${GITHUB_ACTOR:-$USER}" --password-stdin
    fi
}

# Main
main() {
    log_info "MS Framework Docker Build"
    log_info "========================="
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --variant)
                BUILD_VARIANT="$2"
                shift 2
                ;;
            --version)
                VERSION="$2"
                shift 2
                ;;
            --platforms)
                PLATFORMS="$2"
                shift 2
                ;;
            --registry)
                DOCKER_REGISTRY="$2"
                shift 2
                ;;
            --namespace)
                DOCKER_NAMESPACE="$2"
                shift 2
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --help)
                cat << EOF
Usage: $0 [OPTIONS]

Options:
    --variant VARIANT     Build specific variant (minimal|standard|full|debug)
    --version VERSION     Version tag (default: latest)
    --platforms PLATFORMS Platform list (default: linux/amd64,linux/arm64)
    --registry REGISTRY   Docker registry (default: ghcr.io)
    --namespace NAMESPACE Registry namespace (default: mister-smith)
    --dry-run            Build without pushing
    --help               Show this help

Examples:
    $0                              # Build all variants
    $0 --variant standard           # Build standard variant only
    $0 --version v1.0.0 --dry-run  # Test build with version tag
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
    
    # Check prerequisites
    check_buildx
    
    # Setup builder
    setup_builder
    
    # Login to registry
    if [[ "${DRY_RUN:-false}" != "true" ]]; then
        login_registry
    fi
    
    # Build images
    local failed
    build_all_variants
    failed=$?
    
    # Generate compose file
    generate_compose_file
    
    # Summary
    echo
    log_info "Build Summary:"
    log_info "=============="
    log_info "Registry: $DOCKER_REGISTRY/$DOCKER_NAMESPACE/$IMAGE_NAME"
    log_info "Version: $VERSION"
    log_info "Platforms: $PLATFORMS"
    
    if [[ $failed -eq 0 ]]; then
        log_info "All images built successfully! 🐳"
        
        if [[ "${DRY_RUN:-false}" == "true" ]]; then
            log_warn "Dry run - images were not pushed"
        else
            log_info "Images have been pushed to registry"
        fi
    else
        log_error "$failed variant(s) failed to build"
        exit 1
    fi
}

# Run main
main "$@"