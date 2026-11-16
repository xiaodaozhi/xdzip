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
  const [left, top, width, height] = spec.screenshot_artwork.capture_crop;
  if (width < 1366 || height < 768) throw Error('Store screenshot crop is below the required dimensions');
  const inputs = [];
  // Preflight every source before replacing the current image set.
  for (const [locale, copy] of Object.entries(spec.locales)) {
    for (const promotion of copy.promotions) {
      const sourceName = `${locale}-${promotion.id}.jpg`;
      const bytes = fs.readFileSync(path.join(directory, sourceName));
      const metadata = await sharp(bytes).metadata();
      const caption = copy.screenshot_captions[promotion.id];
      if (metadata.format !== 'jpeg' || left < 0 || top < 0 || left + width > metadata.width || top + height > metadata.height) throw Error(`Invalid capture or crop: ${sourceName}`);
      if (!caption || caption.length > 200) throw Error(`Invalid caption: ${sourceName}`);
      inputs.push({locale, scene:promotion.id, sourceName, bytes, metadata, caption});
    }
  }
  const images = [];
  for (const input of inputs) {
    const base = input.locale === 'en-US' ? 'docs/market/windows' : 'docs/zh-CN/market/windows';
    const file = `${base}/images/screenshots/${input.scene}.png`;
    const destination = path.join(root, file);
    fs.mkdirSync(path.dirname(destination), {recursive:true});
    const cropped = sharp(input.bytes).extract({left,top,width,height});
    const pixels = await cropped.clone().removeAlpha().raw().toBuffer();
    const output = await cropped.png({compressionLevel:9}).toBuffer();
    const actual = await sharp(output).removeAlpha().raw().toBuffer();
    if (!actual.equals(pixels)) throw Error(`PNG conversion changed decoded UI pixels: ${input.sourceName}`);
    fs.writeFileSync(destination, output);
    images.push({file, locale:input.locale, scene:input.scene, width, height, caption:input.caption,
      sha256:hash(output), source_file:input.sourceName, source_sha256:hash(input.bytes),
      source_dimensions:[input.metadata.width,input.metadata.height], crop:[left,top,width,height], pixels_sha256:hash(actual)});
  }
  const inventory = {schema_version:1, baseline_revision:spec.baseline.revision,
    package_sha256:spec.baseline.package_sha256, runtime_exe_sha256:spec.baseline.runtime_exe_sha256,
    processing:'Crop the bottom status area, then convert the captured JPEG to PNG without redrawing UI pixels.', images};
  fs.writeFileSync(path.join(root, 'specs/marketing/windows-store-captures.json'), JSON.stringify(inventory,null,2)+'\n');
  console.log(`Imported ${images.length} app screenshots; decoded crop pixels preserved exactly.`);
}

main().catch(error => {console.error(error.message); process.exitCode=1;});
