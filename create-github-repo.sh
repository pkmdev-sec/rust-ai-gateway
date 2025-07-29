#!/bin/bash

# 🚀 GitHub Repository Creation Script
# This script will create a private GitHub repository and push your code

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Rust AI Gateway - GitHub Repository Setup${NC}"
echo "=================================================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -f "README.md" ]; then
    echo -e "${RED}❌ Error: Please run this script from the ai_gateway_router directory${NC}"
    exit 1
fi

# Check if git is initialized
if [ ! -d ".git" ]; then
    echo -e "${RED}❌ Error: Git repository not initialized${NC}"
    exit 1
fi

# Repository configuration
REPO_NAME="rust-ai-gateway"
REPO_DESCRIPTION="Ultra-fast Rust AI Gateway with 11x performance improvement and 100% reliability"
REPO_PRIVATE="true"

echo -e "${YELLOW}📋 Repository Configuration:${NC}"
echo "   Name: $REPO_NAME"
echo "   Description: $REPO_DESCRIPTION"
echo "   Private: $REPO_PRIVATE"
echo ""

# Check if GitHub CLI is installed
if ! command -v gh &> /dev/null; then
    echo -e "${YELLOW}⚠️  GitHub CLI (gh) is not installed.${NC}"
    echo "   Installing GitHub CLI..."
    
    # Install GitHub CLI on macOS
    if command -v brew &> /dev/null; then
        brew install gh
    else
        echo -e "${RED}❌ Please install Homebrew first: https://brew.sh${NC}"
        echo "   Then run: brew install gh"
        exit 1
    fi
fi

# Check if user is logged in to GitHub CLI
if ! gh auth status &> /dev/null; then
    echo -e "${YELLOW}🔐 Please log in to GitHub CLI...${NC}"
    gh auth login
fi

# Get GitHub username
GITHUB_USER=$(gh api user --jq '.login')
echo -e "${GREEN}✅ Logged in as: $GITHUB_USER${NC}"

# Check if repository already exists
if gh repo view "$GITHUB_USER/$REPO_NAME" &> /dev/null; then
    echo -e "${YELLOW}⚠️  Repository $GITHUB_USER/$REPO_NAME already exists.${NC}"
    read -p "Do you want to push to the existing repository? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${RED}❌ Aborted by user${NC}"
        exit 1
    fi
    REPO_EXISTS=true
else
    REPO_EXISTS=false
fi

# Create repository if it doesn't exist
if [ "$REPO_EXISTS" = false ]; then
    echo -e "${BLUE}🏗️  Creating GitHub repository...${NC}"
    
    gh repo create "$REPO_NAME" \
        --description "$REPO_DESCRIPTION" \
        --private \
        --clone=false
    
    echo -e "${GREEN}✅ Repository created successfully!${NC}"
fi

# Add remote if not exists
if ! git remote get-url origin &> /dev/null; then
    echo -e "${BLUE}🔗 Adding GitHub remote...${NC}"
    git remote add origin "https://github.com/$GITHUB_USER/$REPO_NAME.git"
else
    echo -e "${YELLOW}ℹ️  Remote 'origin' already exists${NC}"
fi

# Push to GitHub
echo -e "${BLUE}📤 Pushing code to GitHub...${NC}"

# Check if main branch exists on remote
if git ls-remote --heads origin main | grep -q main; then
    echo -e "${YELLOW}ℹ️  Main branch exists on remote, pushing changes...${NC}"
    git push origin main
else
    echo -e "${BLUE}🌟 Pushing initial commit to main branch...${NC}"
    git push -u origin main
fi

# Set repository topics
echo -e "${BLUE}🏷️  Setting repository topics...${NC}"
gh repo edit "$GITHUB_USER/$REPO_NAME" \
    --add-topic rust \
    --add-topic ai \
    --add-topic gateway \
    --add-topic openai \
    --add-topic anthropic \
    --add-topic performance \
    --add-topic routing \
    --add-topic opencode

# Enable repository features
echo -e "${BLUE}⚙️  Configuring repository settings...${NC}"
gh repo edit "$GITHUB_USER/$REPO_NAME" \
    --enable-issues \
    --enable-wiki \
    --enable-projects

echo ""
echo -e "${GREEN}🎉 SUCCESS! Your repository is ready!${NC}"
echo "=================================================="
echo -e "${BLUE}📍 Repository URL:${NC} https://github.com/$GITHUB_USER/$REPO_NAME"
echo -e "${BLUE}🔗 Clone URL:${NC} git clone https://github.com/$GITHUB_USER/$REPO_NAME.git"
echo ""
echo -e "${YELLOW}📋 Next Steps:${NC}"
echo "1. Visit your repository: https://github.com/$GITHUB_USER/$REPO_NAME"
echo "2. Review the README.md and documentation"
echo "3. Set up environment variables for deployment"
echo "4. Configure any additional repository settings"
echo ""
echo -e "${GREEN}🚀 Your ultra-fast Rust AI Gateway is now on GitHub!${NC}"

# Open repository in browser (optional)
read -p "Do you want to open the repository in your browser? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    gh repo view "$GITHUB_USER/$REPO_NAME" --web
fi