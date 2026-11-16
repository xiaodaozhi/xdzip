#!/usr/bin/env node
/* Import approved, owned app captures as durable Store image inputs. */
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const sharp = require('sharp');
const root = path.resolve(__dirname, '..');
const spec = JSON.parse(fs.readFileSync(path.join(root, 'specs/marketing/windows-store.json'), 'utf8'));
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

async function main() {
  if (process.argv.length !== 3) throw Error('Usage: node scripts/import-windows-store-captures.cjs CAPTURE_DIRECTORY');
  const directory = path.resolve(process.argv[2]);
  const [width, height] = spec.screenshot_artwork.default_capture_size;
  const stripHeight = spec.screenshot_artwork.cursor_cleanup_height;
  if (stripHeight <= 0 || stripHeight >= height - 128) throw Error('Invalid cursor cleanup strip');
  const inputs = [];
  // Preflight every source before replacing the current image set.
  for (const [locale, copy] of Object.entries(spec.locales)) {
    for (const promotion of copy.promotions) {
      const sourceName = `${locale}-${promotion.id}.jpg`;
      const bottomName = `${locale}-${promotion.id}-bottom.jpg`;
      const bytes = fs.readFileSync(path.join(directory, sourceName));
      const bottomBytes = fs.readFileSync(path.join(directory, bottomName));
      const metadata = await sharp(bytes).metadata();
      const bottomMetadata = await sharp(bottomBytes).metadata();
      const caption = copy.screenshot_captions[promotion.id];
      if (metadata.format !== 'jpeg' || bottomMetadata.format !== 'jpeg' ||
          metadata.width !== width || metadata.height !== height ||
          bottomMetadata.width !== width || bottomMetadata.height !== height) throw Error(`Capture does not use the specified default window size: ${sourceName}`);
      if (!caption || caption.length > 200) throw Error(`Invalid caption: ${sourceName}`);
      // The middle of the view must remain static between the two captures.
      const region = {left:0, top:128, width, height:height - stripHeight - 128};
      const first = await sharp(bytes).extract(region).removeAlpha().raw().toBuffer();
      const second = await sharp(bottomBytes).extract(region).removeAlpha().raw().toBuffer();
      let difference = 0;
      for (let i=0; i<first.length; i++) difference += Math.abs(first[i] - second[i]);
      if (difference / first.length > 0.25) throw Error(`App content changed between cursor-cleanup captures: ${sourceName}`);
      inputs.push({locale, scene:promotion.id, sourceName, bottomName, bytes, bottomBytes, caption});
    }
  }
  const images = [];
  for (const input of inputs) {
    const base = input.locale === 'en-US' ? 'docs/market/windows' : 'docs/zh-CN/market/windows';
    const file = `${base}/images/screenshots/${input.scene}.png`;
    const destination = path.join(root, file);
    fs.mkdirSync(path.dirname(destination), {recursive:true});
    const pixels = await sharp(input.bytes).removeAlpha().raw().toBuffer();
    const bottom = await sharp(input.bottomBytes).extract({left:0, top:height - stripHeight, width, height:stripHeight}).removeAlpha().raw().toBuffer();
    // Both regions come directly from the same unchanged application view.
    bottom.copy(pixels, (height - stripHeight) * width * 3);
    const output = await sharp(pixels, {raw:{width,height,channels:3}}).png({compressionLevel:9}).toBuffer();
    const actual = await sharp(output).removeAlpha().raw().toBuffer();
    if (!actual.equals(pixels)) throw Error(`PNG conversion changed captured UI pixels: ${input.sourceName}`);
    fs.writeFileSync(destination, output);
    images.push({file, locale:input.locale, scene:input.scene, width, height, caption:input.caption,
      sha256:hash(output), source_file:input.sourceName, source_sha256:hash(input.bytes),
      bottom_source_file:input.bottomName, bottom_source_sha256:hash(input.bottomBytes),
      source_dimensions:[width,height], cursor_cleanup_strip:[0,height - stripHeight,width,stripHeight], pixels_sha256:hash(actual)});
  }
  const inventory = {schema_version:1, baseline_revision:spec.baseline.revision,
    package_sha256:spec.baseline.package_sha256, runtime_exe_sha256:spec.baseline.runtime_exe_sha256,
    window_mode:'default',
    processing:'Preserve the complete default-size window. Remove the capture cursor marker using the bottom strip from a second capture of the same static view. All UI pixels come from captured app images.', images};
  fs.writeFileSync(path.join(root, 'specs/marketing/windows-store-captures.json'), JSON.stringify(inventory,null,2)+'\n');
  console.log(`Imported ${images.length} complete default-window screenshots; static view and captured UI pixels verified.`);
}

main().catch(error => {console.error(error.message); process.exitCode=1;});
