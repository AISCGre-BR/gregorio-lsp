#!/bin/bash

# Build script for Gregorio LSP

set -e

echo "🎵 Building Gregorio LSP..."

# Clean previous build
echo "Cleaning previous build..."
rm -rf dist/

# Compile TypeScript
echo "Compiling TypeScript..."
npx tsc -b

# Make server executable
echo "Making server executable..."
chmod +x dist/server.js

# Add shebang if not present
if ! head -n 1 dist/server.js | grep -q "^#!"; then
  echo "Adding shebang to server.js..."
  echo "#!/usr/bin/env node" | cat - dist/server.js > dist/server.js.tmp
  mv dist/server.js.tmp dist/server.js
  chmod +x dist/server.js
fi

echo "✅ Build complete!"
echo ""
echo "Run 'npm test' to run tests"
echo "Run 'node dist/server.js --stdio' to start the server"
