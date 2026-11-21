/* Shared macOS/Linux marketing export. Original captures remain independent inputs. */
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const sharp = require('./windows-store-sharp.cjs');
const images = require('./windows-store-image-utils.cjs');
const artwork = require('./build-windows-store-assets.cjs');

const ROOT = path.resolve(__dirname, '..');
const hash = data => crypto.createHash('sha256').update(data).digest('hex');
const readJson = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const languageNames = ['en-US', 'zh-CN'];
const scenes = ['01-browse', '02-create', '03-preview', '04-workspace'];

function argumentsFor(platform, args) {
  const result = {platform, captures: null, output: null, include4k: false};
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--4k') result.include4k = true;
    else if (['--output', '--captures'].includes(args[i]) && args[i + 1]) result[args[i].slice(2)] = args[++i];
    else if (args[i] === '--help') {
      console.log(`Usage: node scripts/build-${platform}-store-assets.cjs [--captures DIR] [--output DIR] [--4k]`);
      return null;
    } else throw Error(`Unknown or incomplete argument: ${args[i]}`);
  }
  const stamp = new Date().toISOString().replace(/[:.]/g, '-');
  result.output = path.resolve(result.output || path.join(ROOT, 'build', `${platform}-store-assets`, stamp));
  const relative = path.relative(ROOT, result.output);
  if (!relative || (!relative.startsWith('..' + path.sep) && relative !== '..' &&
      !relative.startsWith('build' + path.sep) && !relative.startsWith('dist' + path.sep))) {
    throw Error('Output must be below build/ or dist/, or outside the repository');
  }
  if (fs.existsSync(result.output)) throw Error('Output already exists');
  if (result.captures) {
    result.captures = fs.realpathSync(result.captures);
    const belowCapture = path.relative(result.captures, result.output);
    if (!belowCapture || (!belowCapture.startsWith('..' + path.sep) && belowCapture !== '..')) {
      throw Error('Output must not be inside the capture source');
    }
  }
  return result;
}

function capturesFor(options) {
  if (!options.captures) return null;
  const manifestFile = path.join(options.captures, 'capture-session.json');
  if (fs.lstatSync(manifestFile).isSymbolicLink()) throw Error('Capture receipt cannot be a symlink');
  const session = readJson(manifestFile);
  if (session.schema_version !== 1 || session.platform !== options.platform || session.complete !== true ||
      session.provenance !== 'operator-supplied' || !Array.isArray(session.images) || session.images.length !== 8 ||
      !session.identity || typeof session.identity.version !== 'string' || !Number.isInteger(session.identity.build) ||
      typeof session.identity.dirty !== 'boolean' || !/^[a-f0-9]{12}$/.test(session.identity.revision) ||
      !/^[a-f0-9]{64}$/.test(session.identity.executable_sha256) ||
      (session.identity.source_revision !== null &&
        (!/^[a-f0-9]{40}$/.test(session.identity.source_revision) ||
         !session.identity.source_revision.startsWith(session.identity.revision)))) {
    throw Error('Incomplete or wrong-platform capture session');
  }
  const byScene = new Map();
  for (const entry of session.images) {
    const key = `${entry.language}/${entry.scene}`;
    if (!languageNames.includes(entry.language) || !scenes.includes(entry.scene) || byScene.has(key) ||
        entry.file !== `${entry.language}/${entry.scene}.png` || !/^[a-f0-9]{64}$/.test(entry.sha256)) {
      throw Error('Unexpected or duplicate capture identity');
    }
    const file = path.join(options.captures, entry.file);
    const real = fs.realpathSync(file);
    const relative = path.relative(options.captures, real);
    if (!relative || relative.startsWith('..' + path.sep) || path.isAbsolute(relative) || fs.lstatSync(file).isSymbolicLink()) {
      throw Error('Capture escapes its source directory');
    }
    if (fs.statSync(file).size > 50_000_000) throw Error(`Capture is too large: ${key}`);
    const bytes = fs.readFileSync(file);
    if (hash(bytes) !== entry.sha256) throw Error(`Capture digest differs: ${key}`);
    byScene.set(key, {entry, bytes});
  }
  for (const language of languageNames) for (const scene of scenes)
    if (!byScene.has(`${language}/${scene}`)) throw Error('Capture scenes are incomplete');
  return {session, byScene, manifest: fs.readFileSync(manifestFile)};
}

async function build(platform, args = process.argv.slice(2)) {
  if (!['macos', 'linux'].includes(platform)) throw Error('Unsupported platform');
  const options = argumentsFor(platform, args);
  if (!options) return;
  const config = readJson(path.join(ROOT, 'specs', 'marketing', `${platform}-store.json`));
  if (config.schema_version !== 1 || config.platform !== platform ||
      JSON.stringify(config.scenes) !== JSON.stringify(scenes) ||
      JSON.stringify(Object.keys(config.locales).sort()) !== JSON.stringify([...languageNames].sort()) ||
      config.visual_source !== 'specs/marketing/windows-store.json') throw Error('Invalid platform marketing configuration');
  const visual = readJson(path.join(ROOT, config.visual_source));
  const captures = capturesFor(options);
  const sizes = options.include4k ? [[1920, 1080], [3840, 2160]] : [[1920, 1080]];
  const outputParent = path.dirname(options.output);
  let ancestor = outputParent;
  while (!fs.existsSync(ancestor)) ancestor = path.dirname(ancestor);
  if (fs.realpathSync(ancestor) !== ancestor) throw Error('Output parent must not traverse a symlink');
  fs.mkdirSync(outputParent, {recursive: true});
  const stage = fs.mkdtempSync(path.join(outputParent, '.marketing-export-'));
  const entries = [];
  async function save(name, bytes, role, language = 'shared', scene = null) {
    const file = path.join(stage, name);
    fs.mkdirSync(path.dirname(file), {recursive: true});
    fs.writeFileSync(file, bytes, {flag: 'wx'});
    entries.push({file: name, role, language, scene, bytes: bytes.length, sha256: hash(bytes)});
  }
  async function renderSvg(svg) { return sharp(Buffer.from(svg)).png({compressionLevel: 9}).toBuffer(); }
  try {
    if (captures) await save('capture-source/capture-session.json', captures.manifest, 'capture-receipt');
    for (const language of languageNames) {
      const locale = config.locales[language];
      if (!locale || locale.promotions.length !== scenes.length ||
          JSON.stringify(locale.promotions.map(p => p.id)) !== JSON.stringify(scenes) ||
          scenes.some(scene => !locale.screenshot_captions[scene]) || !locale.description ||
          !locale.short_description || !Array.isArray(locale.features) || locale.features.length !== 10 ||
          !Array.isArray(locale.keywords) || !locale.copyright) throw Error('Incomplete platform copy');
      await save(`${language}/text/scenes.json`, Buffer.from(JSON.stringify({platform, language,
        promotions: locale.promotions, screenshot_captions: locale.screenshot_captions}, null, 2) + '\n'), 'copy', language);
      for (const [name, value] of Object.entries({
        'description.txt': locale.description,
        'short-description.txt': locale.short_description,
        'features.txt': locale.features.join('\n'),
        'keywords.txt': locale.keywords.join('\n'),
        'copyright.txt': locale.copyright,
      })) await save(`${language}/text/${name}`, Buffer.from(value + '\n'), 'copy', language);
      await save(`${language}/text/fields.json`, Buffer.from(JSON.stringify({product_name: locale.product_name,
        system_requirements: locale.system_requirements, privacy_url: locale.privacy_url, links: config.links}, null, 2) + '\n'), 'copy', language);
      for (const [index, scene] of scenes.entries()) {
        const input = captures?.byScene.get(`${language}/${scene}`);
        let capture = {cutout_width: visual.screenshot_artwork.frame[2], cutout_height: visual.screenshot_artwork.frame[3], corner_radius: 8};
        if (input) {
          const info = await sharp(input.bytes, {limitInputPixels: 40_000_000}).metadata();
          if (info.format !== 'png' || info.width !== input.entry.width || info.height !== input.entry.height ||
              !info.width || !info.height) throw Error(`Invalid PNG dimensions: ${language}/${scene}`);
          capture = {cutout_width: info.width, cutout_height: info.height, corner_radius: 8};
          await save(`capture-source/${language}/${scene}.png`, input.bytes, 'original-capture', language, scene);
          await save(`${language}/screenshots/${scene}.png`, input.bytes, 'screenshot', language, scene);
        }
        for (const [width, height] of sizes) {
          const background = await renderSvg(artwork.screenshotPromo(locale, index, capture, width, height, images));
          const suffix = `${scene}-${width}x${height}.png`;
          await save(`${language}/backgrounds/${suffix}`, background, 'background', language, scene);
          await save(`${language}/feature-illustrations/${suffix}`, await renderSvg(artwork.promo(locale, index, width, height)), 'feature-illustration', language, scene);
          if (input) {
            const box = images.placement(visual, capture, width, height);
            const layer = await images.screenshotLayer(input.bytes, box);
            const composed = await images.composite(background, layer, box.imageX, box.imageY);
            await images.assertOverlay(composed, layer, box.imageX, box.imageY, background);
            await save(`${language}/promotional/${suffix}`, composed, 'combined-artwork', language, scene);
          }
        }
      }
      for (const [width, height] of [[720, 1080], [1440, 2160]])
        await save(`${language}/branding/poster-${width}x${height}.png`, await renderSvg(artwork.poster(locale, width, height)), 'poster', language);
      for (const size of [1080, 2160])
        await save(`${language}/branding/box-art-${size}.png`, await renderSvg(artwork.square(locale, size, size)), 'box-art', language);
      for (const size of [71, 150, 300, 512, 1024])
        await save(`${language}/branding/${size <= 300 ? 'app-icon' : 'avatar'}-${size}.png`, await renderSvg(artwork.icon(size)), 'brand-icon', language);
      for (const [width, height] of sizes)
        await save(`${language}/branding/hero-${width}x${height}.png`, await renderSvg(artwork.hero(width, height)), 'hero', language);
    }
    const manifest = {schema_version: 1, platform, purpose: config.purpose,
      status: captures ? 'operator-supplied-captures' : 'backgrounds-only',
      provenance_note: 'Screenshot subject, scene and application identity require operator review; no native capture is inferred from PNG bytes.',
      capture: captures?.session.identity ?? null,
      config_sha256: hash(fs.readFileSync(path.join(ROOT, 'specs', 'marketing', `${platform}-store.json`))),
      visual_sha256: hash(fs.readFileSync(path.join(ROOT, config.visual_source))), assets: entries};
    await save('manifest.json', Buffer.from(JSON.stringify(manifest, null, 2) + '\n'), 'manifest');
    const sums = entries.map(e => `${e.sha256}  ${e.file}\n`).sort().join('');
    fs.writeFileSync(path.join(stage, 'SHA256SUMS.txt'), sums, {flag: 'wx'});
    fs.renameSync(stage, options.output);
    console.log(`${platform}: exported ${entries.length} assets (${manifest.status}).`);
    const relativeOutput = path.relative(ROOT, options.output);
    console.log(`Output: ${relativeOutput.startsWith('..') ? 'operator-specified external directory' : relativeOutput}`);
  } finally {
    if (fs.existsSync(stage)) fs.rmSync(stage, {recursive: true, force: true});
  }
}

module.exports = {build};
