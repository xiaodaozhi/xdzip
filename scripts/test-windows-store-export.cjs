/* Exercise the actual importer and export with synthetic images, not native captures. */
const fs = require('node:fs');
const path = require('node:path');
const {spawnSync} = require('node:child_process');
const assert = require('node:assert/strict');
const sharp = require('./windows-store-sharp.cjs');
const {hash} = require('./windows-store-image-utils.cjs');
const root = path.resolve(__dirname, '..');
const spec = require('../specs/marketing/windows-store.json');
const scenes = ['01-welcome', '02-create', '03-preview', '04-workspace'];
function run(command, args) {
  const result = spawnSync(command, args, {cwd: root, encoding: 'utf8'});
  assert.equal(result.status, 0, result.stderr);
}
(async () => {
  fs.mkdirSync(path.join(root, 'build'), {recursive:true});
  const base = fs.mkdtempSync(path.join(root, 'build', 'test-windows-store-export-'));
  try {
    const stage = path.join(base, 'archive');
    fs.mkdirSync(path.join(stage, 'specs/marketing'), {recursive:true});
    fs.writeFileSync(path.join(stage, 'specs/marketing/windows-store.json'), JSON.stringify(spec));
    const image = await sharp({create:{width:320,height:200,channels:3,background:'#e5caa0'}}).png().toBuffer();
    const directories = [];
    for (const language of ['en-US', 'zh-CN']) {
      const directory = path.join(base, language);
      directories.push(directory);
      fs.mkdirSync(directory);
      const images = [];
      for (const scene of scenes) {
        const file = scene + '.png', window_file = scene + '-window.png';
        const header = scene === '02-create' ? 32 : 0;
        const windowImage = header ? await sharp({create:{width:320,height:232,channels:3,background:'#eeeeee'}})
          .composite([{input:image,left:0,top:header}]).png().toBuffer() : image;
        fs.writeFileSync(path.join(directory, file), image);
        fs.writeFileSync(path.join(directory, window_file), windowImage);
        images.push({scene,file,window_file,sha256:hash(image),window_sha256:hash(windowImage),width:320,height:200,dpi:96,
          window_dimensions:[320,200+header],client_crop:[0,header,320,200]});
      }
      fs.writeFileSync(path.join(directory, 'capture-session.json'), JSON.stringify({complete:true,language,
        baseline_revision:spec.baseline.revision, runtime_exe_sha256:spec.baseline.runtime_exe_sha256,
        pinned_store_candidate:true,images}));
    }
    run(process.execPath, ['scripts/import-windows-store-captures.cjs', ...directories, '--root', stage]);
    const inventory = JSON.parse(fs.readFileSync(path.join(stage, 'specs/marketing/windows-store-captures.json')));
    for (const capture of inventory.images.filter(item => item.scene === '02-create')) {
      assert.equal(capture.native_height, 232);
      assert.equal(capture.source_client_crop[1], 32);
      assert.equal(capture.source_sha256, capture.source_window_sha256);
      assert.notEqual(capture.source_client_sha256, capture.source_sha256);
    }
    const output = path.join(base, 'export');
    run('python3', ['scripts/package-windows-store-materials.py', '--from-archive', stage, '--output', output]);
    const manifest = JSON.parse(fs.readFileSync(path.join(output, 'manifest.json')));
    assert.deepEqual(manifest.captured_languages, ['en-US','zh-CN']);
    assert.equal(fs.readFileSync(path.join(output, 'ja-JP/review/runFullTrust.txt'), 'utf8').trim(), spec.locales['ja-JP'].review_notes.runFullTrust);
    for (const language of Object.keys(spec.locales)) {
      assert.ok(fs.existsSync(path.join(output, language, 'text/description.txt')));
      assert.ok(fs.existsSync(path.join(output, language, 'feature-illustrations/01-welcome-1920x1080.png')));
      assert.equal(fs.existsSync(path.join(output, language, 'promotional/01-welcome-1920x1080.png')), ['en-US','zh-CN'].includes(language));
    }
    const receiptFile = path.join(directories[0], 'capture-session.json');
    const receipt = JSON.parse(fs.readFileSync(receiptFile));
    const creation = receipt.images.find(item => item.scene === '02-create');
    fs.writeFileSync(path.join(directories[0], creation.window_file), image);
    creation.window_sha256 = hash(image);
    creation.window_dimensions = [320, 200];
    creation.client_crop = [0, 0, 320, 200];
    fs.writeFileSync(receiptFile, JSON.stringify(receipt));
    const rejectedStage = path.join(base, 'missing-titlebar');
    fs.mkdirSync(path.join(rejectedStage, 'specs/marketing'), {recursive:true});
    fs.writeFileSync(path.join(rejectedStage, 'specs/marketing/windows-store.json'), JSON.stringify(spec));
    const rejected = spawnSync(process.execPath, ['scripts/import-windows-store-captures.cjs', ...directories, '--root', rejectedStage], {cwd: root, encoding: 'utf8'});
    assert.notEqual(rejected.status, 0);
    assert.match(rejected.stderr, /does not include a titlebar/);
    assert.ok(!fs.existsSync(path.join(rejectedStage, 'specs/marketing/windows-store-captures.json')));
    console.log('Windows capture import and ten-language export passed (synthetic fixtures).');
  } finally { fs.rmSync(base, {recursive:true,force:true}); }
})().catch(error => {console.error(error);process.exitCode=1});
