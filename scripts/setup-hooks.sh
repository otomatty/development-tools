#!/bin/sh
#
# Setup script for git hooks
# Run this once after cloning the repository
#

echo "🔧 Setting up git hooks..."

# Configure git to use .githooks directory
git config core.hooksPath .githooks

# Make hooks executable
chmod +x .githooks/*

echo "✅ Git hooks configured!"
echo ""
echo "The following hooks are now active:"
ls -la .githooks/
echo ""
echo "To disable hooks temporarily, use:"
echo "  git commit --no-verify"
