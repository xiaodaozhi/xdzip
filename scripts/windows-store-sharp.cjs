const path = require('node:path');

const toolchain = path.join(__dirname, '..', 'build', 'toolchain', 'windows-store-tools');
let sharpPath;
try {
  sharpPath = require.resolve('sharp', {paths: [toolchain]});
} catch (error) {
  if (error.code !== 'MODULE_NOT_FOUND') throw error;
  try {
    sharpPath = require.resolve('sharp');
  } catch (fallbackError) {
    if (fallbackError.code !== 'MODULE_NOT_FOUND') throw fallbackError;
    throw new Error('Install the Store image dependency with: npm install --prefix build/toolchain/windows-store-tools --no-save --no-package-lock sharp@0.34.5');
  }
}

module.exports = require(sharpPath);
