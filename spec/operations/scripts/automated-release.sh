#!/bin/bash
# automated-release.sh - Automated release process for MS Framework
# This script handles the complete release workflow including version bumping,
# changelog generation, tagging, and artifact creation

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
RELEASE_DIR="$PROJECT_ROOT/releases"
CHANGELOG_FILE="$PROJECT_ROOT/CHANGELOG.md"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

# Version types
VERSION_MAJOR="major"
VERSION_MINOR="minor"
VERSION_PATCH="patch"
VERSION_PRERELEASE="prerelease"

# Parse command line arguments
RELEASE_TYPE="${1:-patch}"
DRY_RUN="${2:-false}"
SKIP_TESTS="${3:-false}"

# Validate release type
if [[ ! "$RELEASE_TYPE" =~ ^(major|minor|patch|prerelease)$ ]]; then
    log_error "Invalid release type: $RELEASE_TYPE"
    echo "Usage: $0 [major|minor|patch|prerelease] [dry-run] [skip-tests]"
    exit 1
fi

# Get current version from Cargo.toml
get_current_version() {
    grep -E '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | cut -d'"' -f2
}

# Bump version based on type
bump_version() {
    local current="$1"
    local type="$2"
    
    # Parse version components
    IFS='.' read -r major minor patch <<< "${current%-*}"
    prerelease="${current#*-}"
    
    if [[ "$current" == "$prerelease" ]]; then
        prerelease=""
    fi
    
    case "$type" in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            prerelease=""
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            prerelease=""
            ;;
        patch)
            patch=$((patch + 1))
            prerelease=""
            ;;
        prerelease)
            if [[ -z "$prerelease" ]]; then
                prerelease="alpha.1"
            else
                # Increment prerelease number
                pre_type="${prerelease%.*}"
                pre_num="${prerelease##*.}"
                prerelease="$pre_type.$((pre_num + 1))"
            fi
            ;;
    esac
    
    # Construct new version
    local new_version="$major.$minor.$patch"
    if [[ -n "$prerelease" ]]; then
        new_version="$new_version-$prerelease"
    fi
    
    echo "$new_version"
}

# Update version in all relevant files
update_version_files() {
    local new_version="$1"
    
    log_info "Updating version to $new_version in all files..."
    
    # Update Cargo.toml
    sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" "$PROJECT_ROOT/Cargo.toml"
    
    # Update Cargo.lock
    cargo update --workspace
    
    # Update README if it contains version badge
    if grep -q "version-.*-" "$PROJECT_ROOT/README.md"; then
        sed -i.bak "s/version-[0-9.]*-/version-$new_version-/" "$PROJECT_ROOT/README.md"
    fi
    
    # Clean up backup files
    find "$PROJECT_ROOT" -name "*.bak" -delete
}

# Generate changelog entry
generate_changelog() {
    local version="$1"
    local date=$(date +%Y-%m-%d)
    
    log_info "Generating changelog for version $version..."
    
    # Get commits since last tag
    local last_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    local commit_range="${last_tag:+$last_tag..HEAD}"
    
    # Create temporary changelog entry
    local temp_changelog=$(mktemp)
    
    cat > "$temp_changelog" << EOF
## [$version] - $date

### Added
EOF
    
    # Parse commits and categorize them
    git log $commit_range --pretty=format:"%s|%h" | while IFS='|' read -r message hash; do
        case "$message" in
            feat:*|feature:*)
                echo "- ${message#*:} ($hash)" >> "$temp_changelog"
                ;;
        esac
    done
    
    cat >> "$temp_changelog" << EOF

### Changed
EOF
    
    git log $commit_range --pretty=format:"%s|%h" | while IFS='|' read -r message hash; do
        case "$message" in
            refactor:*|perf:*|improve:*)
                echo "- ${message#*:} ($hash)" >> "$temp_changelog"
                ;;
        esac
    done
    
    cat >> "$temp_changelog" << EOF

### Fixed
EOF
    
    git log $commit_range --pretty=format:"%s|%h" | while IFS='|' read -r message hash; do
        case "$message" in
            fix:*|bugfix:*)
                echo "- ${message#*:} ($hash)" >> "$temp_changelog"
                ;;
        esac
    done
    
    cat >> "$temp_changelog" << EOF

### Security
EOF
    
    git log $commit_range --pretty=format:"%s|%h" | while IFS='|' read -r message hash; do
        case "$message" in
            security:*|sec:*)
                echo "- ${message#*:} ($hash)" >> "$temp_changelog"
                ;;
        esac
    done
    
    # Prepend to existing changelog
    if [[ -f "$CHANGELOG_FILE" ]]; then
        echo -e "\n" >> "$temp_changelog"
        cat "$CHANGELOG_FILE" >> "$temp_changelog"
    fi
    
    mv "$temp_changelog" "$CHANGELOG_FILE"
}

# Run pre-release checks
run_pre_release_checks() {
    log_step "Running pre-release checks..."
    
    # Check for uncommitted changes
    if [[ -n $(git status --porcelain) ]]; then
        log_error "Uncommitted changes detected. Please commit or stash them."
        exit 1
    fi
    
    # Ensure we're on main branch
    local current_branch=$(git rev-parse --abbrev-ref HEAD)
    if [[ "$current_branch" != "main" ]] && [[ "$current_branch" != "master" ]]; then
        log_warn "Not on main branch. Current branch: $current_branch"
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    # Run tests unless skipped
    if [[ "$SKIP_TESTS" != "true" ]]; then
        log_step "Running tests..."
        cargo test --all-features
        cargo clippy --all-targets --all-features -- -D warnings
        cargo audit
    else
        log_warn "Skipping tests as requested"
    fi
}

# Build release artifacts
build_release_artifacts() {
    local version="$1"
    
    log_step "Building release artifacts for version $version..."
    
    # Create release directory
    mkdir -p "$RELEASE_DIR/$version"
    
    # Build for all targets
    local targets=(
        "x86_64-unknown-linux-gnu"
        "x86_64-unknown-linux-musl"
        "aarch64-unknown-linux-gnu"
        "x86_64-apple-darwin"
        "aarch64-apple-darwin"
        "x86_64-pc-windows-gnu"
    )
    
    for target in "${targets[@]}"; do
        log_info "Building for $target..."
        
        if command -v cross &> /dev/null; then
            cross build --release --target "$target" --features tier_3
        else
            cargo build --release --target "$target" --features tier_3 || log_warn "Failed to build for $target"
        fi
        
        # Package the binary
        if [[ -f "target/$target/release/mister-smith-framework" ]]; then
            tar czf "$RELEASE_DIR/$version/mister-smith-$version-$target.tar.gz" \
                -C "target/$target/release" mister-smith-framework \
                -C "$PROJECT_ROOT" README.md LICENSE-MIT LICENSE-APACHE
        elif [[ -f "target/$target/release/mister-smith-framework.exe" ]]; then
            # Windows executable
            zip -j "$RELEASE_DIR/$version/mister-smith-$version-$target.zip" \
                "target/$target/release/mister-smith-framework.exe" \
                README.md LICENSE-MIT LICENSE-APACHE
        fi
    done
    
    # Build WASM targets
    log_info "Building WASM targets..."
    cargo build --release --target wasm32-unknown-unknown --features wasm
    cargo build --release --target wasm32-wasi --features wasm
    
    # Package WASM artifacts
    if [[ -d "target/wasm32-unknown-unknown/release" ]]; then
        tar czf "$RELEASE_DIR/$version/mister-smith-$version-wasm.tar.gz" \
            -C target/wasm32-unknown-unknown/release . \
            -C target/wasm32-wasi/release .
    fi
    
    # Generate checksums
    cd "$RELEASE_DIR/$version"
    sha256sum * > SHA256SUMS
    cd - > /dev/null
}

# Create git tag and push
create_release_tag() {
    local version="$1"
    
    log_step "Creating git tag for version $version..."
    
    # Create annotated tag
    git tag -a "v$version" -m "Release version $version"
    
    if [[ "$DRY_RUN" != "true" ]]; then
        git push origin "v$version"
    else
        log_warn "Dry run: Would push tag v$version"
    fi
}

# Publish to crates.io
publish_to_crates() {
    local version="$1"
    
    log_step "Publishing version $version to crates.io..."
    
    if [[ "$DRY_RUN" != "true" ]]; then
        cargo publish --features tier_3
    else
        log_warn "Dry run: Would publish to crates.io"
        cargo publish --dry-run --features tier_3
    fi
}

# Create GitHub release
create_github_release() {
    local version="$1"
    
    log_step "Creating GitHub release for version $version..."
    
    # Check if gh CLI is available
    if ! command -v gh &> /dev/null; then
        log_warn "GitHub CLI not found. Skipping GitHub release creation."
        return
    fi
    
    # Extract changelog for this version
    local changelog_section=$(awk "/## \[$version\]/{flag=1;next}/## \[/{flag=0}flag" "$CHANGELOG_FILE")
    
    if [[ "$DRY_RUN" != "true" ]]; then
        gh release create "v$version" \
            --title "Release v$version" \
            --notes "$changelog_section" \
            "$RELEASE_DIR/$version"/*
    else
        log_warn "Dry run: Would create GitHub release v$version"
    fi
}

# Main release process
main() {
    log_info "Starting release process for MS Framework"
    log_info "Release type: $RELEASE_TYPE"
    log_info "Dry run: $DRY_RUN"
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Get current version
    CURRENT_VERSION=$(get_current_version)
    log_info "Current version: $CURRENT_VERSION"
    
    # Calculate new version
    NEW_VERSION=$(bump_version "$CURRENT_VERSION" "$RELEASE_TYPE")
    log_info "New version: $NEW_VERSION"
    
    # Run pre-release checks
    run_pre_release_checks
    
    # Update version files
    if [[ "$DRY_RUN" != "true" ]]; then
        update_version_files "$NEW_VERSION"
    else
        log_warn "Dry run: Would update version files to $NEW_VERSION"
    fi
    
    # Generate changelog
    if [[ "$DRY_RUN" != "true" ]]; then
        generate_changelog "$NEW_VERSION"
    else
        log_warn "Dry run: Would generate changelog for $NEW_VERSION"
    fi
    
    # Commit version bump
    if [[ "$DRY_RUN" != "true" ]]; then
        git add -A
        git commit -m "chore: bump version to $NEW_VERSION"
        git push origin HEAD
    else
        log_warn "Dry run: Would commit version bump"
    fi
    
    # Build release artifacts
    build_release_artifacts "$NEW_VERSION"
    
    # Create git tag
    create_release_tag "$NEW_VERSION"
    
    # Publish to crates.io
    publish_to_crates "$NEW_VERSION"
    
    # Create GitHub release
    create_github_release "$NEW_VERSION"
    
    log_info "Release process completed successfully!"
    log_info "Version $NEW_VERSION has been released"
    
    # Show release artifacts
    log_info "Release artifacts created:"
    ls -la "$RELEASE_DIR/$NEW_VERSION"
}

# Run main function
main