/* Shared macOS/Linux marketing export. Original captures remain independent inputs. */
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const sharp = require('./windows-store-sharp.cjs');
const images = require('./windows-store-image-utils.cjs');
const artwork = require('./build-windows-store-assets.cjs');
const {frameClient} = require('./marketing-window-frame.cjs');
const crossPlatform = require('./marketing-platform-composite.cjs');
const frames = require('../specs/marketing/cross-platform-frames.json');
const {main: mainLayout} = require('../specs/visual/layout-scale.json');

const ROOT = path.resolve(__dirname, '..');
const hash = data => crypto.createHash('sha256').update(data).digest('hex');
const readJson = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const languageNames = readJson(path.join(ROOT, 'resources/ui-languages.json')).languages.map(item => item.id);
const scenes = ['01-welcome', '02-create', '03-preview', '04-workspace'];
const outputScale = readJson(path.join(ROOT, 'specs/capture/marketing-screenshot-scale.json')).physical_pixels_per_qt_logical_pixel;

function opaqueCornerRadius(platform, scene) {
  if (platform === 'macos')
    return scene === '02-create' ? frames.source.opaque_operation_radius : frames.source.opaque_main_radius;
  if (scene === '02-create') return frames.targets[platform].operation_radius;
  return platform === 'linux' ? mainLayout.linux_frame.cornerRadius : frames.targets.windows.main_radius;
}

function argumentsFor(platform, args) {
  const result = {platform, captures: null, compositeFromMacos: null, alternateMacos: null, compositionLayout: null, output: null,
    include4k: false, createClientOnly: false, createWindow: false};
  const valueArgs = {'--output': 'output', '--captures': 'captures',
    '--composite-from-macos': 'compositeFromMacos', '--alternate-macos': 'alternateMacos', '--composition-layout': 'compositionLayout'};
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--4k') result.include4k = true;
    else if (args[i] === '--create-client-only') result.createClientOnly = true;
    else if (args[i] === '--create-window') result.createWindow = true;
    else if (Object.hasOwn(valueArgs, args[i]) && args[i + 1]) result[valueArgs[args[i]]] = args[++i];
    else if (args[i] === '--help') {
      console.log(`Usage: node scripts/build-${platform}-store-assets.cjs [--captures DIR] [--output DIR] [--4k] [--create-client-only | --create-window]`);
      if (platform !== 'macos')
        console.log('       --composite-from-macos DIR --composition-layout FILE [--alternate-macos DIR] [--output DIR] [--4k]');
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
  if (result.createClientOnly && result.createWindow) throw Error('Choose one creation window scope');
  if ((result.createClientOnly || result.createWindow) && !result.captures) throw Error('Creation window scope requires captures');
  if (Boolean(result.compositeFromMacos) !== Boolean(result.compositionLayout) ||
      (result.alternateMacos && !result.compositeFromMacos) ||
      (result.compositeFromMacos && (platform === 'macos' || result.captures)))
    throw Error('Use a measured macOS composite source and layout only for Windows or Linux, separately from native captures');
  for (const source of ['captures', 'compositeFromMacos', 'alternateMacos']) {
    if (!result[source]) continue;
    result[source] = fs.realpathSync(result[source]);
    const belowCapture = path.relative(result[source], result.output);
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
  const sourceScale = session.reported_native_scale;
  if (sourceScale !== undefined && (!Number.isFinite(sourceScale) || sourceScale < outputScale || sourceScale > 4)) {
    throw Error('Invalid reported native capture scale');
  }
  if (session.schema_version !== 1 || session.platform !== options.platform || session.complete !== true ||
      session.provenance !== 'operator-supplied' || !Array.isArray(session.images) || !session.images.length || session.images.length % scenes.length !== 0 ||
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
  const capturedLanguages = new Set(session.images.map(entry => entry.language));
  for (const language of capturedLanguages) for (const scene of scenes)
    if (!byScene.has(`${language}/${scene}`)) throw Error('Capture scenes are incomplete');
  return {session, byScene, manifest: fs.readFileSync(manifestFile)};
}

async function build(platform, args = process.argv.slice(2)) {
  if (!['windows', 'macos', 'linux'].includes(platform)) throw Error('Unsupported platform');
  const options = argumentsFor(platform, args);
  if (!options) return;
  if (platform === 'windows' && options.captures) throw Error('Use the Windows verified capture export workflow for screenshots');
  const config = readJson(path.join(ROOT, 'specs', 'marketing', `${platform}-store.json`));
  if (config.schema_version !== 1 || config.platform !== platform ||
      JSON.stringify(config.scenes) !== JSON.stringify(scenes) ||
      JSON.stringify(Object.keys(config.locales).sort()) !== JSON.stringify([...languageNames].sort()) ||
      config.visual_source !== 'specs/marketing/windows-store.json') throw Error('Invalid platform marketing configuration');
  const visual = readJson(path.join(ROOT, config.visual_source));
  const captures = capturesFor(options.compositeFromMacos
    ? {...options, platform: 'macos', captures: options.compositeFromMacos} : options);
  const alternate = options.alternateMacos
    ? capturesFor({...options, platform: 'macos', captures: options.alternateMacos}) : null;
  const composition = options.compositeFromMacos
    ? crossPlatform.readComposition(options.compositionLayout, captures, languageNames, alternate) : null;
  if (captures && !composition && !options.createClientOnly && !options.createWindow && captures.session.capture_method !== 'interactive-window')
    throw Error('Declare whether 02-create is client-only (--create-client-only) or already includes the titlebar (--create-window)');
  const sizes = options.include4k ? [[1920, 1080], [3840, 2160]] : [[1920, 1080]];
  const outputParent = path.dirname(options.output);
  let ancestor = outputParent;
  while (!fs.existsSync(ancestor)) ancestor = path.dirname(ancestor);
  if (fs.realpathSync(ancestor) !== ancestor) throw Error('Output parent must not traverse a symlink');
  fs.mkdirSync(outputParent, {recursive: true});
  const stage = fs.mkdtempSync(path.join(outputParent, '.marketing-export-'));
  const entries = [];
  const transformations = [];
  async function save(name, bytes, role, language = 'shared', scene = null) {
    const file = path.join(stage, name);
    fs.mkdirSync(path.dirname(file), {recursive: true});
    fs.writeFileSync(file, bytes, {flag: 'wx'});
    entries.push({file: name, role, language, scene, bytes: bytes.length, sha256: hash(bytes)});
  }
  async function renderSvg(svg) { return sharp(Buffer.from(svg)).png({compressionLevel: 9}).toBuffer(); }
  try {
    if (captures && !composition) await save('capture-source/capture-session.json', captures.manifest, 'capture-receipt');
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
        const key = `${language}/${scene}`;
        const selectedCapture = composition?.alternateScenes.has(key) ? alternate : captures;
        const input = selectedCapture?.byScene.get(key);
        let capture;
        let presentationBytes = input?.bytes;
        if (input) {
          const info = await sharp(input.bytes, {limitInputPixels: 40_000_000}).metadata();
          if (info.format !== 'png' || info.width !== input.entry.width || info.height !== input.entry.height ||
              !info.width || !info.height) throw Error(`Invalid PNG dimensions: ${language}/${scene}`);
          let outWidth = info.width, outHeight = info.height;
          const sourceScale = captures.session.reported_native_scale;
          const file = `${language}/screenshots/${scene}.png`;
          if (composition) {
            const {png, ...receipt} = await crossPlatform.compose(input, composition, platform, language, scene);
            presentationBytes = png;
            outWidth = receipt.width; outHeight = receipt.height;
            transformations.push({...receipt, scene, file,
              source_revision: selectedCapture.session.identity.source_revision,
              source_receipt_sha256: selectedCapture === alternate ? composition.alternateDigest : composition.sourceDigest});
          } else if (sourceScale > outputScale) {
            const ratio = outputScale / sourceScale;
            outWidth = Math.max(1, Math.round(info.width * ratio));
            outHeight = Math.max(1, Math.round(info.height * ratio));
            presentationBytes = await sharp(input.bytes).resize(outWidth, outHeight, {fit: 'fill', kernel: 'lanczos3'}).png().toBuffer();
            transformations.push({method: 'native-capture-downsample', scene, file, source_scale: sourceScale,
              output_scale: outputScale, source_width: info.width, source_height: info.height,
              width: outWidth, height: outHeight, source_sha256: input.entry.sha256, sha256: hash(presentationBytes)});
          }
          if (scene === '02-create' && options.createClientOnly) {
            const framed = await frameClient(presentationBytes, platform, language);
            presentationBytes = framed.png;
            outWidth = framed.width; outHeight = framed.height;
            const {png, ...receipt} = framed;
            transformations.push({...receipt, scene, file});
          }
          capture = {cutout_width: outWidth, cutout_height: outHeight};
          if (!composition) await save(`capture-source/${language}/${scene}.png`, input.bytes, 'original-capture', language, scene);
          await save(`${language}/screenshots/${scene}.png`, presentationBytes,
            composition || (scene === '02-create' && options.createClientOnly) ? 'composite-screenshot' : 'screenshot', language, scene);
        }
        for (const [width, height] of sizes) {
          const background = await renderSvg(artwork.screenshotPromo(locale, index, width, height));
          const suffix = `${scene}-${width}x${height}.png`;
          await save(`${language}/backgrounds/${suffix}`, background, 'background', language, scene);
          await save(`${language}/feature-illustrations/${suffix}`, await renderSvg(artwork.promo(locale, index, width, height)), 'feature-illustration', language, scene);
          if (input) {
            const box = images.placement(visual, capture, width, height);
            const layer = await images.promotionalLayer(presentationBytes, box, opaqueCornerRadius(platform, scene),
              visual.screenshot_artwork, {width, height});
            const composed = await images.composite(background, layer.png, layer.left, layer.top);
            await images.assertOverlay(composed, layer.png, layer.left, layer.top, background);
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
    const capture = composition ? {source_platform: 'macos', version: captures.session.identity.version,
      build: captures.session.identity.build, source_revision: captures.session.identity.source_revision,
      source_receipt_sha256: composition.sourceDigest, layout_sha256: composition.digest, template_sha256: composition.templateDigest,
      ...(alternate ? {alternate_source_revision: alternate.session.identity.source_revision,
        alternate_source_receipt_sha256: composition.alternateDigest} : {})}
      : captures?.session.identity ?? null;
    const manifest = {schema_version: 1, platform, purpose: config.purpose,
      captured_languages: captures ? [...new Set(captures.session.images.map(entry => entry.language))] : [],
      status: composition ? 'cross-platform-composites'
        : captures ? (options.createClientOnly ? 'operator-supplied-with-composites' : 'operator-supplied-captures') : 'backgrounds-only',
      provenance_note: composition
        ? 'Target-framed illustrations derived from reviewed macOS renders for presentation. Source files remain with the original Mac capture.'
        : 'Screenshot subject, scene and application identity require operator review; synthetic titlebars are composites, not native captures.',
      capture, transformations,
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

module.exports = {build, capturesFor};
