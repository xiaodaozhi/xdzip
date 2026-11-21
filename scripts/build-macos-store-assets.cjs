#!/usr/bin/env node
require('./build-platform-store-assets.cjs').build('macos').catch(error => { console.error(error.message); process.exitCode = 1; });
