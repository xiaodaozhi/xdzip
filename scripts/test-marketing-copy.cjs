/* Check language coverage and locale-specific typography without desktop or native images. */
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const artwork = require('./build-windows-store-assets.cjs');
const languages = require('../resources/ui-languages.json').languages.map(item => item.id).sort();
const scenes = ['01-welcome', '02-create', '03-preview', '04-workspace'];
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
    const readme = language === 'en-US' ? 'README.md' : `README.${language}.md`;
    const tagline = /<p align="center"><strong>(.*?)<\/strong><\/p>/.exec(fs.readFileSync(path.join(__dirname, '..', readme), 'utf8'))?.[1];
    const expected = language === 'zh-CN' ? '压缩和解压，都简单一点。' : tagline.replace(/[。.]?$/u, language === 'zh-TW' || language === 'ja-JP' ? '。' : '.');
    assert.ok(expected, `${language} product tagline`);
    assert.equal(copy.promotions[0].headline.join('').replace(/\s/gu, ''), expected.replace(/\s/gu, ''), `${platform} ${language} welcome tagline`);

    for (let index = 0; index < scenes.length; index++) {
      assert.ok(copy.promotions[index].headline.at(-1).endsWith(['zh-CN','zh-TW','ja-JP'].includes(language) ? '。' : '.'), `${platform} ${language} punctuation`);
      assert.ok(copy.screenshot_captions[scenes[index]], language);
      const svg = artwork.screenshotPromo(copy, index, 1920, 1080);
      assert.ok(!svg.includes('capture-shadow'), 'The base must exclude the screenshot silhouette');
      assert.ok(svg.includes(`font-family="${copy.font_family}"`), `${platform} ${language} font`);
      assert.ok(!svg.includes('undefined'), `${platform} ${language} missing artwork text`);
    }
  }
}
console.log('Ten-language copy, scenes and locale-specific font selection passed.');
