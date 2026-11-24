/* End-to-end export checks use synthetic PNG fixtures, never product screenshots. */
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const assert = require('node:assert/strict');
const {spawnSync} = require('node:child_process');
const sharp = require('./windows-store-sharp.cjs');
const root = path.resolve(__dirname, '..');
const languages = require('../resources/ui-languages.json').languages.map(item => item.id);
const scenes = ['01-welcome', '02-create', '03-preview', '04-workspace'];
const hash = data => crypto.createHash('sha256').update(data).digest('hex');
const {readComposition, compose} = require('./marketing-platform-composite.cjs');
const readPixel = async (png, left, top) =>
  [...await sharp(png).extract({left, top, width: 1, height: 1}).ensureAlpha().raw().toBuffer()];
const run = (platform, args, success = true) => {
  const result = spawnSync(process.execPath, [path.join(root, 'scripts', `build-${platform}-store-assets.cjs`), ...args], {encoding: 'utf8'});
  if (success) assert.equal(result.status, 0, result.stderr);
  else assert.notEqual(result.status, 0, 'invalid input should fail');
};

async function checkCrossPlatform(base) {
  const regions = {shortcuts: [[539, 656, 131, 30], [1163, 656, 130, 30]], path: [228, 942, 510, 40]};
  const receipt = Buffer.from('test capture receipt');
  const source = {manifest: receipt, session: {platform: 'macos', reported_native_scale: 2,
    identity: {source_revision: 'a'.repeat(40), dirty: false}, images: Array(40).fill(null)}};
  const profile = path.join(base, 'regions.json');
  const map = {schema_version: 1, source_receipt_sha256: hash(receipt),
    regions: Object.fromEntries(languages.map(language => [language, regions]))};
  fs.writeFileSync(profile, JSON.stringify(map));
  const composition = readComposition(profile, source, languages);
  assert.throws(() => readComposition(profile, {...source, manifest: Buffer.from('unrelated')}, languages), /measured region map/);
  const alternateReceipt = Buffer.from('alternate receipt');
  const alternate = {...source, manifest: alternateReceipt, byScene: new Map([['en-US/04-workspace', {}]])};
  map.alternate_capture = {source_receipt_sha256: hash(alternateReceipt), scenes: [{language: 'en-US', scene: '04-workspace'}]};
  fs.writeFileSync(profile, JSON.stringify(map));
  assert.throws(() => readComposition(profile, source, languages), /Declare both/);
  assert.deepEqual([...readComposition(profile, source, languages, alternate).alternateScenes], ['en-US/04-workspace']);
  map.alternate_capture.scenes.push({language: 'en-US', scene: '01-welcome'});
  fs.writeFileSync(profile, JSON.stringify(map));
  assert.throws(() => readComposition(profile, source, languages, alternate), /Invalid or duplicate/);

  async function input(width, height, layers) {
    const svg = Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}">${layers}</svg>`);
    const bytes = await sharp(svg).png().toBuffer();
    return {bytes, entry: {width, height, sha256: hash(bytes)}};
  }
  const main = await input(2048, 1244, '<rect width="2048" height="1244" fill="white"/>' +
    '<rect width="2048" height="96" fill="#F7F8FA"/><rect x="16" y="24" width="36" height="36" fill="red"/>' +
    '<rect x="430" y="530" width="520" height="200" fill="#0058FF"/>' +
    '<rect x="550" y="666" width="18" height="13" fill="white"/><rect x="1170" y="666" width="18" height="13" fill="#626B86"/>' +
    '<rect x="800" y="900" width="80" height="80" fill="#669933"/>');
  const create = await input(982, 1536, '<rect width="982" height="1536" fill="white"/>' +
    '<rect width="982" height="64" fill="red"/><rect x="245" y="953" width="255" height="15" fill="#141827"/>');
  const derived = [];
  for (const target of ['windows', 'linux']) {
    const welcome = await compose(main, composition, target, 'en-US', '01-welcome');
    assert.deepEqual([welcome.width, welcome.height, welcome.output_scale], [1536, 933, 1.5]);
    assert.equal(welcome.method, 'cross-platform-composite');
    assert.equal(welcome.target_platform, target);
    assert.equal(welcome.source_sha256, main.entry.sha256);
    assert.deepEqual(await readPixel(welcome.png, 32, 33), [247, 248, 250, 255]);
    assert.deepEqual(await readPixel(welcome.png, 626, 700), [102, 153, 51, 255]);
    derived.push(welcome);
    const dialog = await compose(create, composition, target, 'en-US', '02-create');
    assert.deepEqual([dialog.width, dialog.height, dialog.header_pixels], [737, 1152, 48]);
    assert.equal(dialog.titlebar, 'synthetic-native-decoration');
    assert.notDeepEqual(await readPixel(dialog.png, 280, 33), [255, 0, 0, 255]);
  }
  assert.notEqual(derived[0].sha256, derived[1].sha256);
  const invalid = {...composition, regions: {'en-US': {...regions, shortcuts: [[0, 0, 8, 8], regions.shortcuts[1]]}}};
  await assert.rejects(compose(main, invalid, 'windows', 'en-US', '01-welcome'), /outside/);
  const occluded = await input(2048, 1244, '<rect width="2048" height="1244" fill="white"/>' +
    '<rect x="400" y="152" width="260" height="60" fill="#142036"/>');
  await assert.rejects(compose(occluded, composition, 'windows', 'en-US', '04-workspace'), /breadcrumb tooltip/);
}

(async () => {
  fs.mkdirSync(path.join(root, 'build'), {recursive: true});
  const base = fs.mkdtempSync(path.join(root, 'build', 'test-platform-export-'));
  try {
    await checkCrossPlatform(base);
    const fixture = await sharp({create: {width: 800, height: 500, channels: 4, background: '#abcdef'}}).png().toBuffer();
    for (const platform of ['macos', 'linux']) {
      const input = path.join(base, `${platform}-captures`);
      const records = [];
      for (const lang of languages) for (const scene of scenes) {
        const name = `${lang}/${scene}.png`;
        fs.mkdirSync(path.join(input, lang), {recursive: true});
        fs.writeFileSync(path.join(input, name), fixture);
        records.push({language: lang, scene, file: name, width: 800, height: 500, sha256: hash(fixture)});
      }
      const session = {schema_version: 1, platform, complete: true, provenance: 'operator-supplied', capture_method: 'interactive-window',
        identity: {version: 'test', build: 1, revision: 'a'.repeat(12), source_revision: 'a'.repeat(40), dirty: false, executable_sha256: 'b'.repeat(64)}, images: records};
      fs.writeFileSync(path.join(input, 'capture-session.json'), JSON.stringify(session));
      const readCapture = require('./build-platform-store-assets.cjs').capturesFor;
      const subset = {...session, images: session.images.filter(entry => entry.language === 'ja-JP')};
      fs.writeFileSync(path.join(input, 'capture-session.json'), JSON.stringify(subset));
      assert.equal(readCapture({platform, captures: input}).byScene.size, 4);
      subset.images.pop();
      fs.writeFileSync(path.join(input, 'capture-session.json'), JSON.stringify(subset));
      assert.throws(() => readCapture({platform, captures: input}), /Incomplete|incomplete/);
      const framedSession = {...session, capture_method:'existing-png', images: session.images.filter(entry => entry.language === 'ja-JP')};
      fs.writeFileSync(path.join(input, 'capture-session.json'), JSON.stringify(framedSession));
      run(platform, ['--captures', input, '--output', path.join(base, `${platform}-undeclared`)], false);
      const framedOutput = path.join(base, `${platform}-framed`);
      run(platform, ['--captures', input, '--output', framedOutput, '--create-client-only']);
      const frameManifest = JSON.parse(fs.readFileSync(path.join(framedOutput, 'manifest.json')));
      assert.equal(frameManifest.status, 'operator-supplied-with-composites');
      assert.equal(frameManifest.transformations.length, 1);
      assert.equal(frameManifest.transformations[0].method, 'synthetic-operation-titlebar');
      const framed = fs.readFileSync(path.join(framedOutput, 'ja-JP/screenshots/02-create.png'));
      const frameInfo = await sharp(framed).metadata();
      assert.deepEqual([frameInfo.width, frameInfo.height], [800, 548]);
      assert.deepEqual(await sharp(framed).extract({left:0,top:48,width:800,height:500}).ensureAlpha().raw().toBuffer(),
                       await sharp(fixture).ensureAlpha().raw().toBuffer());
      assert.deepEqual(fs.readFileSync(path.join(framedOutput, 'capture-source/ja-JP/02-create.png')), fixture);
      const scaledSession = {...framedSession, reported_native_scale: 2};
      fs.writeFileSync(path.join(input, 'capture-session.json'), JSON.stringify(scaledSession));
      const scaledOutput = path.join(base, `${platform}-scaled`);
      run(platform, ['--captures', input, '--output', scaledOutput, '--create-window']);
      const scaledMeta = await sharp(path.join(scaledOutput, 'ja-JP/screenshots/01-welcome.png')).metadata();
      assert.deepEqual([scaledMeta.width, scaledMeta.height], [600, 375]);
      assert.deepEqual(fs.readFileSync(path.join(scaledOutput, 'capture-source/ja-JP/01-welcome.png')), fixture);
      fs.writeFileSync(path.join(input, 'capture-session.json'), JSON.stringify(session));
      const output = path.join(base, `${platform}-export`);
      run(platform, ['--captures', input, '--output', output]);
      const manifest = JSON.parse(fs.readFileSync(path.join(output, 'manifest.json')));
      assert.equal(manifest.platform, platform);
      assert.equal(manifest.status, 'operator-supplied-captures');
      const banners = manifest.assets.filter(asset => asset.role === 'featured-banner');
      assert.equal(banners.length, platform === 'linux' ? 1 : 0);
      if (banners.length) {
        const [banner] = banners;
        assert.equal(banner.file, 'snap-store/featured-banner.png');
        assert.equal(banner.language, 'shared');
        const {format, width, height} = await sharp(path.join(output, banner.file)).metadata();
        assert.deepEqual([format, width, height], ['png', 1440, 480]);
        assert.ok(banner.bytes <= 2_000_000);
      }
      for (const lang of languages) for (const scene of scenes) {
        assert.deepEqual(fs.readFileSync(path.join(output, lang, 'screenshots', `${scene}.png`)), fixture);
        assert.deepEqual(fs.readFileSync(path.join(output, 'capture-source', lang, `${scene}.png`)), fixture);
        for (const type of ['backgrounds', 'promotional', 'feature-illustrations'])
          assert.ok(fs.existsSync(path.join(output, lang, type, `${scene}-1920x1080.png`)));
      }
      const lines = fs.readFileSync(path.join(output, 'SHA256SUMS.txt'), 'utf8').trim().split('\n');
      for (const line of lines) {
        const match = /^([0-9a-f]{64})  (.+)$/.exec(line);
        assert.ok(match);
        assert.equal(hash(fs.readFileSync(path.join(output, match[2]))), match[1]);
      }
      assert.equal(lines.length, manifest.assets.length + 1);
      run(platform, ['--captures', input, '--output', output], false);
      session.platform = platform === 'macos' ? 'linux' : 'macos';
      fs.writeFileSync(path.join(input, 'capture-session.json'), JSON.stringify(session));
      run(platform, ['--captures', input, '--output', path.join(base, platform + '-wrong')], false);
      session.platform = platform;
      session.images[0].sha256 = '0'.repeat(64);
      fs.writeFileSync(path.join(input, 'capture-session.json'), JSON.stringify(session));
      run(platform, ['--captures', input, '--output', path.join(base, platform + '-tampered')], false);
      assert.ok(!fs.existsSync(path.join(base, platform + '-tampered')));
    }
    const output = path.join(base, 'background-only');
    run('macos', ['--output', output, '--4k']);
    const manifest = JSON.parse(fs.readFileSync(path.join(output, 'manifest.json')));
    assert.equal(manifest.status, 'backgrounds-only');
    assert.equal(manifest.capture, null);
    assert.ok(fs.existsSync(path.join(output, 'zh-CN/backgrounds/01-welcome-3840x2160.png')));
    assert.ok(!fs.existsSync(path.join(output, 'zh-CN/promotional')));
    console.log('Platform export tests passed (synthetic fixtures).');
  } finally { fs.rmSync(base, {recursive: true, force: true}); }
})().catch(error => { console.error(error); process.exitCode = 1; });
