/* Adapt the measured, light-theme macOS demo scenes to target-platform chrome. */
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const sharp = require('./windows-store-sharp.cjs');
const {frameClient} = require('./marketing-window-frame.cjs');
const layout = require('../specs/visual/layout-scale.json');
const reference = require('../specs/visual/qt-main-reference.json');
const scale = require('../specs/capture/marketing-screenshot-scale.json').physical_pixels_per_qt_logical_pixel;
const frames = require('../specs/marketing/cross-platform-frames.json');
const configFile = require.resolve('../specs/marketing/cross-platform-frames.json');
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const xml = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('"', '&quot;');
const sourceScale = 2;
const atSource = logical => Math.round(logical * sourceScale);
const artwork = name => path.join(__dirname, '..', 'design', 'assets', 'png', 'light', name + '@2x.png');

function isCleanMacCapture(captures) {
  const {session} = captures;
  return session.platform === frames.source.platform && session.identity.source_revision &&
    !session.identity.dirty && session.reported_native_scale === sourceScale;
}

function readComposition(file, captures, languages, alternate = null) {
  const bytes = fs.readFileSync(file);
  const data = JSON.parse(bytes);
  const sourceDigest = hash(captures.manifest);
  if (data.schema_version !== frames.schema_version || data.source_receipt_sha256 !== sourceDigest ||
      !isCleanMacCapture(captures) ||
      captures.session.images.length !== languages.length * 4 || !data.regions ||
      Object.keys(data.regions).sort().join() !== [...languages].sort().join())
    throw Error('Composition requires a measured region map for this complete, clean 200% macOS capture');
  for (const regions of Object.values(data.regions)) {
    if (!Array.isArray(regions.shortcuts) || regions.shortcuts.length !== 2 || !Array.isArray(regions.path))
      throw Error('Expected two shortcut rectangles and the destination field rectangle');
  }
  const replacement = data.alternate_capture;
  const alternateDigest = alternate ? hash(alternate.manifest) : null;
  const alternateScenes = new Set();
  if (Boolean(alternate) !== Boolean(replacement)) throw Error('Declare both the alternate capture and its scene selection');
  if (alternate) {
    if (!isCleanMacCapture(alternate) || alternate.session.identity.version !== captures.session.identity.version ||
        alternate.session.identity.build !== captures.session.identity.build ||
        replacement.source_receipt_sha256 !== alternateDigest || !Array.isArray(replacement.scenes) || !replacement.scenes.length)
      throw Error('Alternate scenes require a clean macOS receipt for the same product version and build');
    for (const item of replacement.scenes) {
      const key = `${item.language}/${item.scene}`;
      if (!languages.includes(item.language) || !['03-preview', '04-workspace'].includes(item.scene) ||
          alternateScenes.has(key) || !alternate.byScene.has(key))
        throw Error('Invalid or duplicate alternate archive scene');
      alternateScenes.add(key);
    }
  }
  return {regions: data.regions, digest: hash(bytes), templateDigest: hash(fs.readFileSync(configFile)),
    sourceDigest, alternateDigest, alternateScenes};
}

function svg(width, height, contents) {
  return Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}">${contents}</svg>`);
}

function pixel(raw, x, y) {
  const start = (y * raw.info.width + x) * 4;
  return raw.data.subarray(start, start + 4);
}

function equalColor(pixel, expected, tolerance) {
  return pixel.every((channel, index) => Math.abs(channel - expected[index]) <= tolerance);
}

function rejectBreadcrumbTooltip(raw) {
  // The demo breadcrumb can show the archive name in a dark tooltip over the toolbar.
  // A clean toolbar can have dark icon and text pixels but has no solid dark patch here.
  const area = {left: 396, top: 149, width: 285, height: 67};
  let ink = 0;
  for (let y = area.top; y < area.top + area.height; y++) {
    for (let x = area.left; x < area.left + area.width; x++) {
      const point = pixel(raw, x, y);
      if (point[0] < 50 && point[1] < 60 && point[2] < 85) ink++;
    }
  }
  if (ink > 2000) throw Error('Archive scene is obscured by a breadcrumb tooltip');
}

function fillMacCorners(raw, background) {
  const {width, height} = raw.info;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const point = pixel(raw, x, y);
      if (point[3] === 255) continue;
      if (Math.min(x, width - x - 1) > 64 || Math.min(y, height - y - 1) > 64)
        throw Error('Unexpected transparency inside the source window');
      point.set(background);
    }
  }
}

function rawImage(raw) {
  return sharp(raw.data, {raw: {width: raw.info.width, height: raw.info.height, channels: 4}});
}

function patchText(raw, rectangle, replacement, style) {
  const [x, y, width, height] = rectangle;
  if (rectangle.length !== 4 || !rectangle.every(Number.isInteger) || width <= 0 || height <= 0 ||
      x < 0 || y < 0 || x + width > raw.info.width || y + height > raw.info.height ||
      x < style.zone[0] || y < style.zone[1] || x + width > style.zone[2] || y + height > style.zone[3])
    throw Error('Measured rectangle lies outside the supported scene area');
  const color = pixel(raw, x, y);
  if (!equalColor(color, style.background, 2))
    throw Error('Measured text region does not have its expected background');
  let ink = 0;
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      if (equalColor(pixel(raw, x + col, y + row), color, 2)) continue;
      if (!row || row === height - 1 || !col || col === width - 1)
        throw Error('Measured text region clips adjacent content');
      ink++;
    }
  }
  if (ink < 25) throw Error('Expected source text inside the measured rectangle');
  const fill = `rgb(${color[0]},${color[1]},${color[2]})`;
  const image = svg(width, height, `<rect width="${width}" height="${height}" fill="${fill}"/>` +
    `<text x="${style.inset}" y="${style.baseline}" font-family="${xml(style.font)}" font-size="${style.fontSize}" fill="${style.color}" opacity="${style.opacity}">${xml(replacement)}</text>`);
  return {input: image, left: x, top: y};
}

async function mainTitle(raw, platform, scene) {
  const {main} = layout;
  const {width} = raw.info;
  const height = atSource(main.titlebar.height - 1);
  const background = [247, 248, 250, 255];
  if (!equalColor(pixel(raw, atSource(700), 4), background, 2))
    throw Error('Expected the unobstructed light-theme macOS titlebar');
  const den = main.source_to_logical[0] / main.source_to_logical[1];
  const g = reference.source_geometry;
  const brandSource = g.brandMacX * den;
  const brandTarget = 20 * main.density;
  const tabsSource = g.tabFirstMacX * den;
  const tabsTarget = g.tabFirstOtherX * den;
  const count = frames.source.main_tab_counts[scene];
  const tabsWidth = count * g.tabWidth * den + count * main.titlebar.tab_gap + 36 * main.density;
  const settingsWidth = g.settingsSize * den;
  const settingsSource = main.logical_window[0] - g.settingsRightInset * den - settingsWidth;
  const linux = platform === 'linux';
  const buttonWidth = linux ? main.linux_frame.captionWidth : main.interaction.captionButtonWidth;
  const rightInset = linux ? main.linux_frame.captionRightInset : 0;
  const offset = rightInset + buttonWidth * 3;
  async function move(sourceX, size, targetX) {
    const crop = {left: atSource(sourceX), top: 0, width: atSource(size), height};
    return {input: await rawImage(raw).extract(crop).png().toBuffer(), left: atSource(targetX), top: 0};
  }
  const margin = 3;
  const settingsTarget = settingsSource - offset;
  if (tabsTarget + tabsWidth + margin >= settingsTarget - margin)
    throw Error('Target window cannot retain the measured tab widths');
  const clientItems = await Promise.all([
    move(brandSource - margin, tabsSource - brandSource - margin, brandTarget - margin),
    move(tabsSource - margin, tabsWidth + 2 * margin, tabsTarget - margin),
    move(settingsSource - margin, settingsWidth + 2 * margin, settingsTarget - margin),
  ]);
  const midY = atSource(main.titlebar.height / 2);
  const iconSize = atSource(main.interaction.captionIconSize);
  const icons = await Promise.all(['minimize', 'maximize', 'close'].map(async (name, index) => {
    const x = atSource(main.logical_window[0] - offset + (index + 0.5) * buttonWidth);
    return {x, input: await sharp(artwork(name)).resize(iconSize, iconSize).png().toBuffer(),
      left: x - iconSize / 2, top: midY - iconSize / 2};
  }));
  const circles = linux ? icons.map(({x}) => `<circle cx="${x}" cy="${midY}" r="${atSource(main.linux_frame.captionSize) / 2}" fill="#EDF2FF"/>`).join('') : '';
  const face = svg(width, height, `<rect width="${width}" height="${height}" fill="#F7F8FA"/>${circles}`);
  return sharp(face).composite([...clientItems, ...icons.map(({input, left, top}) => ({input, left, top}))]).png().toBuffer();
}

async function targetFrame(bytes, radius, outline) {
  const {width, height} = await sharp(bytes).metadata();
  const r = radius * scale;
  const mask = svg(width, height, `<rect width="${width}" height="${height}" rx="${r}" fill="white"/>`);
  const line = svg(width, height, `<rect x=".5" y=".5" width="${width - 1}" height="${height - 1}" rx="${r - .5}" fill="none" stroke="${outline}"/>`);
  return sharp(bytes).ensureAlpha().composite([{input: mask, blend: 'dest-in'}, {input: line}])
    .png({compressionLevel: 9}).toBuffer();
}

async function compose(input, composition, platform, language, scene) {
  const target = frames.targets[platform];
  if (!target) throw Error('Unsupported composite target');
  const {entry, bytes} = input;
  const raw = await sharp(bytes, {limitInputPixels: 40_000_000}).ensureAlpha()
    .raw().toBuffer({resolveWithObject: true});
  const decoded = await sharp(bytes).metadata();
  const operation = scene === '02-create';
  const operationHeight = layout.operation.components.createClientHeight;
  const header = atSource(frames.source.operation_titlebar_height);
  const logical = operation ? [layout.operation.logical_widths.create,
    operationHeight + frames.source.operation_titlebar_height] : layout.main.logical_window;
  if (decoded.format !== 'png' || raw.info.width !== entry.width || raw.info.height !== entry.height ||
      entry.width !== atSource(logical[0]) || entry.height !== atSource(logical[1]))
    throw Error(`Unexpected source geometry: ${language}/${scene}`);
  if (scene === '03-preview' || scene === '04-workspace') rejectBreadcrumbTooltip(raw);
  fillMacCorners(raw, [255, 255, 255, 255]);
  const safe = composition.regions[language];
  const layers = [];
  if (operation) {
    const baseline = Math.round(safe.path[3] / 2 + 9);
    layers.push(patchText(raw, safe.path, target.operation_demo_path, {
      zone: [145, 905, 805, 1000], background: [255, 255, 255, 255],
      color: '#141827', opacity: 1, inset: 8, baseline, font: 'Noto Sans, Arial, sans-serif', fontSize: atSource(13),
    }));
  } else {
    layers.push({input: await mainTitle(raw, platform, scene), left: 0, top: 0});
    if (scene === '01-welcome') {
      for (const [index, box] of safe.shortcuts.entries()) {
        layers.push(patchText(raw, box, index ? 'Ctrl+N' : 'Ctrl+O', {
          zone: index ? [1140, 610, 1300, 840] : [520, 610, 680, 840],
          background: index ? [255, 255, 255, 255] : [0, 88, 255, 255],
          color: index ? '#626B86' : '#FFFFFF', opacity: .7,
          inset: 5, baseline: box[3] - 4, font: 'Noto Sans, Arial, sans-serif', fontSize: atSource(11),
        }));
      }
    }
  }
  const patched = await rawImage(raw).composite(layers).png().toBuffer();
  const width = Math.round(logical[0] * scale);
  const clientHeight = Math.round((operation ? operationHeight : logical[1]) * scale);
  let image = sharp(patched);
  if (operation) image = image.extract({left: 0, top: header, width: entry.width, height: entry.height - header});
  const client = await image.resize(width, clientHeight, {fit: 'fill', kernel: 'lanczos3'}).png().toBuffer();
  const framed = operation ? await frameClient(client, platform, language) : {png: client, height: clientHeight};
  const mainRadius = platform === 'linux' ? layout.main.linux_frame.cornerRadius : target.main_radius;
  const png = await targetFrame(framed.png, operation ? target.operation_radius : mainRadius,
    operation ? target.operation_outline : target.main_outline);
  return {png, width, height: framed.height, method: 'cross-platform-composite', source_platform: 'macos',
    target_platform: platform, source_scale: sourceScale, output_scale: scale,
    source_width: entry.width, source_height: entry.height, source_sha256: entry.sha256, sha256: hash(png),
    titlebar: operation ? 'synthetic-native-decoration' : 'application-caption-controls',
    ...(operation ? {demo_path: target.operation_demo_path, path_region: safe.path, header_pixels: framed.header_pixels}
      : scene === '01-welcome' ? {shortcuts: ['Ctrl+O', 'Ctrl+N'], shortcut_regions: safe.shortcuts} : {}),
  };
}

module.exports = {readComposition, compose};
