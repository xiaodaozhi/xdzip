/* Check language coverage and locale-specific typography without desktop or native images. */
const assert = require('node:assert/strict');
const artwork = require('./build-windows-store-assets.cjs');
const languages = require('../resources/ui-languages.json').languages.map(item => item.id).sort();
const scenes = ['01-browse', '02-create', '03-preview', '04-workspace'];
const layout = {placement: () => ({imageX:638,imageY:186,imageWidth:1206,imageHeight:790,scale:1})};
for (const platform of ['windows', 'macos', 'linux']) {
  const config = require(`../specs/marketing/${platform}-store.json`);
  assert.deepEqual(Object.keys(config.locales).sort(), languages);
  for (const [language, copy] of Object.entries(config.locales)) {
    assert.equal(copy.features.length, 10, language);
    assert.ok(copy.description && copy.short_description && copy.font_family, language);
    if (platform === 'windows' && !['en-US', 'zh-CN'].includes(language)) {
      assert.ok(copy.review_notes?.runFullTrust && copy.review_notes?.certification, `${language} review notes`);
    }
    assert.deepEqual(copy.promotions.map(item => item.id), scenes);
    for (let index = 0; index < scenes.length; index++) {
      assert.ok(copy.screenshot_captions[scenes[index]], language);
      const svg = artwork.screenshotPromo(copy, index, {corner_radius:8}, 1920, 1080, layout);
      assert.ok(svg.includes(`font-family="${copy.font_family}"`), `${platform} ${language} font`);
      assert.ok(!svg.includes('undefined'), `${platform} ${language} missing artwork text`);
    }
  }
}
console.log('Ten-language copy, scenes and locale-specific font selection passed.');
