/* Add an explicitly synthetic platform titlebar to a client-only operation-window PNG. */
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const sharp = require('./windows-store-sharp.cjs');
const root = path.resolve(__dirname, '..');
const languages = require('../resources/ui-languages.json').languages.map(item => item.id);
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const escape = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');

async function frameClient(bytes, platform, language) {
  if (!['windows', 'macos', 'linux'].includes(platform) || !languages.includes(language)) throw Error('Unsupported platform or language');
  const info = await sharp(bytes, {limitInputPixels: 40_000_000}).metadata();
  if (info.format !== 'png' || !info.width || !info.height || info.width < 240 || info.width > 5000 || info.height > 5000) throw Error('Expected a bounded client-only PNG');
  const title = require(`../resources/i18n/${language}.json`).messages['ui.fe189b616c1d'];
  const font = require(`../specs/marketing/${platform}-store.json`).locales[language].font_family;
  const width = info.width, height = 48; // 32 logical points at 150%.
  const center = width / 2;
  let buttons;
  if (platform === 'macos') {
    buttons = [24, 58.5, 93].map((x, i) => `<circle cx="${x}" cy="24" r="10.5" fill="${i ? '#D9D9D9' : '#FF5F57'}" stroke="${i ? '#BDBDBD' : '#E14942'}" stroke-width="1"/>`).join('');
  } else if (platform === 'linux') {
    buttons = `<circle cx="${width-27}" cy="24" r="13.5" fill="#E3E3E3"/><path d="M${width-32} 19l10 10m0-10l-10 10" fill="none" stroke="#303030" stroke-width="1.8"/>`;
  } else {
    buttons = `<path d="M${width-42} 16.5l15 15m0-15l-15 15" fill="none" stroke="#20242D" stroke-width="1.5"/>`;
  }
  const titleX = platform === 'windows' ? 45 : center;
  const anchor = platform === 'windows' ? 'start' : 'middle';
  const leftBound = platform === 'macos' ? 120 : platform === 'windows' ? 45 : 56;
  const rightBound = platform === 'windows' ? 70 : platform === 'linux' ? 56 : 12;
  const face = platform === 'windows' ? '#FFFFFF' : platform === 'linux' ? '#F4F4F4' : '#F7F8FA';
  const weight = platform === 'linux' ? 600 : 400;
  const label = `<text x="${titleX}" y="30" text-anchor="${anchor}" font-family="${escape(font)}" font-size="18" font-weight="${weight}" fill="#20242D">${escape(title)}</text>`;
  // Clip long translations to avoid overlapping system buttons.
  const svg = Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}"><rect width="${width}" height="${height}" fill="${face}"/><path d="M0 47.5H${width}" stroke="#D7DCE5"/><clipPath id="title"><rect x="${leftBound}" width="${Math.max(0,width-leftBound-rightBound)}" height="48"/></clipPath><g clip-path="url(#title)">${label}</g>${buttons}</svg>`);
  const layers = [{input:svg,left:0,top:0},{input:bytes,left:0,top:height}];
  if (platform === 'windows') {
    const icon = await sharp(path.join(root,'design','assets','brand','xdzip-128.png')).resize(24,24).png().toBuffer();
    layers.push({input:icon,left:12,top:12});
  }
  const png = await sharp({create:{width,height:info.height+height,channels:4,background:'#00000000'}})
    .composite(layers).png({compressionLevel:9}).toBuffer();
  return {png, width, height:info.height+height, header_pixels:height, source_sha256:sha(bytes), sha256:sha(png),
    method:'synthetic-operation-titlebar', target_platform:platform, language, header_scale:1.5, source_scale_verified:false};
}

module.exports = {frameClient};
