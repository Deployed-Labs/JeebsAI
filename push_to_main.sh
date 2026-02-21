#!/usr/bin/env bash
#
# Complete Git Push to Main - Run this on your LOCAL machine
#
set -e

echo "📦 JeebsAI - Commit and Push to Main"
echo "====================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    error "Not a git repository. Please run 'git init' first."
    exit 1
fi

# Show current branch
CURRENT_BRANCH=$(git branch --show-current)
info "Current branch: $CURRENT_BRANCH"

# Check if we need to switch to main
if [ "$CURRENT_BRANCH" != "main" ] && [ "$CURRENT_BRANCH" != "master" ]; then
    warn "You are on branch '$CURRENT_BRANCH'"
    read -p "Switch to main branch? (y/n) " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Check if main exists
        if git show-ref --verify --quiet refs/heads/main; then
            git checkout main
        elif git show-ref --verify --quiet refs/heads/master; then
            git checkout master
        else
            error "Neither 'main' nor 'master' branch exists"
            exit 1
        fi
        success "Switched to main branch"
    else
        error "Deployment requires main branch. Exiting."
        exit 1
    fi
fi

echo ""

# Show git status
info "Current git status:"
echo ""
git status
echo ""

# Count uncommitted changes
CHANGES=$(git status --porcelain | wc -l)

if [ "$CHANGES" -eq 0 ]; then
    warn "No changes to commit"
    read -p "Push existing commits anyway? (y/n) " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        info "Exiting without pushing"
        exit 0
    fi
else
    info "Found $CHANGES uncommitted change(s)"
    echo ""

    read -p "Add and commit all changes? (y/n) " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Show what will be added
        info "Files to be added:"
        git status --short
        echo ""

        # Add all changes
        info "Adding all changes..."
        git add .

        # Get commit message
        echo ""
        read -p "Enter commit message (or press Enter for default): " commit_message

        if [ -z "$commit_message" ]; then
            commit_message="Deploy: Add learning systems, knowledge retrieval, and proactive proposals - $(date '+%Y-%m-%d %H:%M:%S')"
        fi

        # Commit
        info "Committing with message: $commit_message"
        git commit -m "$commit_message"

        success "✅ Changes committed!"
        echo ""
    else
        warn "Skipping commit. Only existing commits will be pushed."
        echo ""
    fi
fi

# Check for unpushed commits
UNPUSHED=$(git log origin/$(git branch --show-current)..HEAD --oneline 2>/dev/null | wc -l || echo "0")

if [ "$UNPUSHED" -eq 0 ]; then
    warn "No commits to push"
    info "Local and remote are in sync"
    exit 0
else
    info "Found $UNPUSHED commit(s) to push"
    echo ""

    info "Commits to be pushed:"
    git log origin/$(git branch --show-current)..HEAD --oneline --decorate
    echo ""
fi

# Confirm push
read -p "Push to origin/main? (y/n) " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    warn "Push cancelled"
    exit 0
fi

# Push to main
info "Pushing to origin/main..."
echo ""

git push origin main

echo ""
success "=========================================="
success "🎉 Successfully pushed to main!"
success "=========================================="
echo ""

info "Next steps:"
echo "  1. To deploy to VPS, run: ./push_and_deploy.sh"
echo "  2. Or SSH to VPS and run: cd /opt/jeebs && sudo ./deploy_to_vps.sh"
echo ""

# Offer to create a tag
read -p "Create a release tag? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    read -p "Enter tag name (e.g., v2.0.0): " tag_name

    if [ -n "$tag_name" ]; then
        read -p "Enter tag message (or press Enter to skip): " tag_message

        if [ -n "$tag_message" ]; then
            git tag -a "$tag_name" -m "$tag_message"
        else
            git tag "$tag_name"
        fi

        git push origin "$tag_name"
        success "✅ Tag $tag_name created and pushed!"
    fi
fi

echo ""
success "All done! 🚀"
echo ""
