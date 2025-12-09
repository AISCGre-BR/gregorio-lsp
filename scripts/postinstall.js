#!/usr/bin/env node

/**
 * Post-install script to verify dependencies and setup
 */

const fs = require('fs');
const path = require('path');

console.log('🎵 Gregorio LSP - Post-install check\n');

// Check for tree-sitter-gregorio
const treeSitterPath = path.join(__dirname, '..', '..', 'tree-sitter-gregorio');
let hasTreeSitter = false;

try {
  if (fs.existsSync(treeSitterPath)) {
    const packagePath = path.join(treeSitterPath, 'package.json');
    if (fs.existsSync(packagePath)) {
      hasTreeSitter = true;
      console.log('✅ tree-sitter-gregorio found at:', treeSitterPath);
    }
  }
} catch (error) {
  // Ignore
}

if (!hasTreeSitter) {
  console.log('⚠️  tree-sitter-gregorio not found');
  console.log('   The LSP will use the TypeScript fallback parser');
  console.log('   For better performance, install tree-sitter-gregorio:');
  console.log('   cd ../tree-sitter-gregorio && npm install\n');
}

// Check Node version
const nodeVersion = process.version;
const major = parseInt(nodeVersion.slice(1).split('.')[0]);

if (major < 16) {
  console.log('⚠️  Node.js version', nodeVersion, 'detected');
  console.log('   Node.js >= 16.0.0 is recommended\n');
} else {
  console.log('✅ Node.js version:', nodeVersion, '\n');
}

// Success message
console.log('✅ Gregorio LSP installation complete!');
console.log('\nNext steps:');
console.log('  npm run build    - Build the project');
console.log('  npm test         - Run tests');
console.log('  npm run watch    - Watch mode for development');
console.log('\nDocumentation:');
console.log('  README.md        - Main documentation');
console.log('  docs/API.md      - API reference');
console.log('  docs/DEVELOPMENT.md - Development guide');
console.log('\nExamples:');
console.log('  examples/        - Example GABC files\n');
