#!/usr/bin/env node
/* Import complete, lossless native captures, preserving original inputs. */
const fs = require('node:fs');
const path = require('node:path');
const sharp = require('./windows-store-sharp.cjs');
const {hash,makeCutout} = require('./windows-store-image-utils.cjs');
async function main() {
  const rootIndex = process.argv.indexOf('--root');
  if (rootIndex < 3 || rootIndex !== process.argv.length - 2) {
    throw Error('Import captures with refresh-windows-store-screenshots.py or supply an explicit --root staging directory');
  }
  const root = path.resolve(process.argv[rootIndex + 1]);
  if (root === path.resolve(__dirname,'..')) throw Error('Import screenshots into an export staging directory');
  const spec = JSON.parse(fs.readFileSync(path.join(root,'specs/marketing/windows-store.json'),'utf8'));
  const inputs = [];
  const seen = new Set();
  for (const argument of process.argv.slice(2, rootIndex)) {
    const directory = path.resolve(argument);
    const sessionBytes = fs.readFileSync(path.join(directory,'capture-session.json'));
    const session = JSON.parse(sessionBytes.toString('utf8').replace(/^\uFEFF/,''));
    const locale = session.language;
    if (!spec.locales[locale] || seen.has(locale)) throw Error('Unknown or duplicate capture language');
    seen.add(locale);
    if (!session.complete || session.baseline_revision !== spec.baseline.revision ||
        session.runtime_exe_sha256 !== spec.baseline.runtime_exe_sha256 || !session.pinned_store_candidate) throw Error(`Unverified candidate session: ${locale}`);
    const scenes = spec.locales[locale].promotions.map(p=>p.id).sort();
    if (JSON.stringify(session.images.map(c=>c.scene).sort()) !== JSON.stringify(scenes)) throw Error(`Incomplete scene set: ${locale}`);
    for (const source of session.images) {
      for (const file of [source.file,source.window_file]) if (path.basename(file) !== file || !file.endsWith('.png')) throw Error('Invalid native capture filename');
      const bytes = fs.readFileSync(path.join(directory,source.file));
      const windowBytes = fs.readFileSync(path.join(directory,source.window_file));
      if (hash(bytes) !== source.sha256 || hash(windowBytes) !== source.window_sha256) throw Error(`Capture SHA-256 differs: ${source.file}`);
      const meta = await sharp(bytes).metadata();
      const windowMeta = await sharp(windowBytes).metadata();
      if (meta.format !== 'png' || windowMeta.format !== 'png' || meta.width !== source.width || meta.height !== source.height ||
          !Array.isArray(source.window_dimensions) || windowMeta.width !== source.window_dimensions[0] || windowMeta.height !== source.window_dimensions[1]) throw Error(`Original PNG required: ${source.file}`);
      const [left,top,width,height] = source.client_crop;
      const pixels = await sharp(bytes).removeAlpha().raw().toBuffer();
      const windowPixels = await sharp(windowBytes).extract({left,top,width,height}).removeAlpha().raw().toBuffer();
      if (!pixels.equals(windowPixels)) throw Error(`Native client/window pixels differ: ${source.file}`);
      // The creation screenshot is a client crop: use its separately verified
      // native window reference so the system titlebar is retained.
      if (source.scene === '02-create' && top <= 0) throw Error('Creation window reference does not include a titlebar');
      const nativeBytes = source.scene === '02-create' ? windowBytes : bytes;
      const nativeWidth = source.scene === '02-create' ? windowMeta.width : width;
      const nativeHeight = source.scene === '02-create' ? windowMeta.height : height;
      const nativePixels = source.scene === '02-create' ? await sharp(windowBytes).removeAlpha().raw().toBuffer() : pixels;
      const cutout = await makeCutout(nativeBytes,nativeWidth,nativeHeight,source.dpi,spec.screenshot_artwork.cleanup);
      const caption = spec.locales[locale].screenshot_captions[source.scene];
      if (!caption || caption.length > 200) throw Error('Invalid native screenshot caption');
      inputs.push({locale,source,nativeBytes,nativeWidth,nativeHeight,nativePixels,cutout,caption,sessionHash:hash(sessionBytes)});
    }
  }
  const images = [];
  // All source sets have passed before any existing Store input is replaced.
  for (const input of inputs) {
    const {locale,source,nativeBytes,nativeWidth,nativeHeight,nativePixels,cutout,caption,sessionHash} = input;
    const base = locale === 'en-US' ? 'developers/market/windows' : locale === 'zh-CN' ? 'developers/zh-CN/market/windows' : `developers/market/windows/locales/${locale}`;
    const file = `${base}/images/screenshots/${source.scene}.png`;
    const nativeFile = `design/assets/marketing/windows-store-native/${locale}/${source.scene}.png`;
    for (const destination of [file,nativeFile]) fs.mkdirSync(path.dirname(path.join(root,destination)),{recursive:true});
    fs.writeFileSync(path.join(root,nativeFile),nativeBytes);
    fs.writeFileSync(path.join(root,file),cutout.png);
    images.push({file,native_file:nativeFile,cutout_file:file,locale,scene:source.scene,width:cutout.width,height:cutout.height,dpi:source.dpi,
      native_width:nativeWidth,native_height:nativeHeight,
      caption,sha256:hash(cutout.png),pixels_sha256:hash(nativePixels),source_file:source.scene === '02-create' ? source.window_file : source.file,source_sha256:hash(nativeBytes),
      source_client_sha256:source.sha256,
      source_session_sha256:sessionHash,source_window_sha256:source.window_sha256,source_client_crop:source.client_crop,
      cutout_sha256:hash(cutout.png),cutout_width:cutout.width,cutout_height:cutout.height,
      cutout_rgba_sha256:cutout.rgba_sha256,perimeter_crop:cutout.crop,corner_radius:cutout.radius,
      border_pixels:spec.screenshot_artwork.cleanup.border_pixels,border_color:spec.screenshot_artwork.cleanup.border_color});
  }
  const inventory = {schema_version:3,baseline_revision:spec.baseline.revision,package_sha256:spec.baseline.package_sha256,
    runtime_exe_sha256:spec.baseline.runtime_exe_sha256,window_mode:'default',
    processing:'The creation scene uses its native window reference including the titlebar; other scenes use client images. Native sources remain byte-identical in design/assets. Published screenshots remove one top row, clear old desktop colors inside the corner squares, apply corner alpha and add a one-pixel inner border without changing dimensions or alpha. Other cropped RGB pixels remain unchanged. No JPEG or screenshot upscaling.',images};
  fs.writeFileSync(path.join(root,'specs/marketing/windows-store-captures.json'),JSON.stringify(inventory,null,2)+'\n');
  console.log(`Imported ${images.length} lossless native PNGs and transparent cutouts; source hashes and window pixels verified.`);
}
main().catch(error=>{console.error(error.message);process.exitCode=1});
