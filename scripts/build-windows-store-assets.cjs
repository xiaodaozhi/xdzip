#!/usr/bin/env node
/* Render editable vector marketing artwork without launching a desktop or browser. */
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const rootIndex = require.main === module ? process.argv.indexOf('--root') : -1;
const repository = path.resolve(__dirname, '..');
const listingsOnly = rootIndex === -1;
const ROOT = listingsOnly ? repository : path.resolve(process.argv[rootIndex + 1]);
if (!listingsOnly && ROOT === repository) throw Error('Render images in an export staging directory');
const sharp = listingsOnly ? null : require('./windows-store-sharp.cjs');
const imageTools = listingsOnly ? null : require('./windows-store-image-utils.cjs');
const spec = JSON.parse(fs.readFileSync(path.join(ROOT, 'specs/marketing/windows-store.json'), 'utf8'));
const brand = fs.readFileSync(path.join(ROOT, 'design/assets/brand/xdzip.svg'), 'utf8');
const brandUri = 'data:image/svg+xml;base64,' + Buffer.from(brand).toString('base64');
const check = process.argv.includes('--check');
const indexFile = path.join(ROOT, 'specs/marketing/windows-store-assets.json');
const captureFile = path.join(ROOT, 'specs/marketing/windows-store-captures.json');
const captures = listingsOnly ? null : JSON.parse(fs.readFileSync(captureFile, 'utf8'));
const previous = fs.existsSync(indexFile) ? JSON.parse(fs.readFileSync(indexFile, 'utf8')) : null;
const jobs = [];
const largeSizes = process.argv.includes('--4k') ? [[1920,1080],[3840,2160]] : [[1920,1080]];
const hash = data => crypto.createHash('sha256').update(data).digest('hex');
const esc = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');
const v = spec.visual;
let activeFont = v.font;
const svg = (width, height, body, view = '0 0 1920 1080') => `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="${view}"><defs><linearGradient id="paper" x2=".9" y2="1"><stop stop-color="#FFFFFF"/><stop offset="1" stop-color="#E8F0FF"/></linearGradient><linearGradient id="ocean" x2=".85" y2="1"><stop stop-color="#145DE5"/><stop offset=".48" stop-color="#2057FF"/><stop offset="1" stop-color="#142C95"/></linearGradient><linearGradient id="night" x2="1" y2="1"><stop stop-color="#213756"/><stop offset="1" stop-color="#0A1530"/></linearGradient><linearGradient id="sky" x2="0" y2="1"><stop stop-color="#73B7FF"/><stop offset="1" stop-color="#C6ECF4"/></linearGradient><radialGradient id="glow"><stop stop-color="#BBDEFF" stop-opacity=".8"/><stop offset="1" stop-color="#BBDEFF" stop-opacity="0"/></radialGradient><filter id="shadow" x="-40%" y="-40%" width="180%" height="200%"><feDropShadow dx="0" dy="26" stdDeviation="26" flood-color="#203E7B" flood-opacity=".17"/></filter></defs>${body}</svg>`;
const text = (x, y, content, size = 32, color = v.ink, weight = 400, anchor = 'start', maxWidth = Infinity) => {
  const units = [...String(content)].reduce((sum, c) => sum + (/[^\u0000-\u04ff]/u.test(c) ? 1 : .72), 0);
  size = Math.min(size, maxWidth / Math.max(units, 1));
  return `<text x="${x}" y="${y}" font-family="${esc(activeFont)}" font-size="${size}" font-weight="${weight}" fill="${color}" text-anchor="${anchor}">${esc(content)}</text>`;
};
// Keep localized copy readable within the artwork column instead of shrinking long lines.
function wrapped(lines, maxUnits) {
  const result = [];
  for (const line of lines) {
    let remaining = String(line).trim();
    while (remaining) {
      let used = 0, end = 0, space = -1;
      for (const char of remaining) {
        const width = /[^\u0000-\u04ff]/u.test(char) ? 1 : .72;
        if (used + width > maxUnits) break;
        used += width;
        end += char.length;
        if (/\s/u.test(char)) space = end;
      }
      if (end >= remaining.length) { result.push(remaining); break; }
      if (space > 0) end = space;
      if (!end) throw Error('Text column is too narrow');
      result.push(remaining.slice(0, end).trimEnd());
      remaining = remaining.slice(end).trimStart();
    }
  }
  return result;
}
// Use one headline size for the welcome slogan so line breaks retain a consistent hierarchy.
function headlineSize(lines, size, width) {
  const units = line => [...String(line)].reduce((sum, c) => sum + (/[^\u0000-\u04ff]/u.test(c) ? 1 : .72), 0);
  return Math.min(size, ...lines.map(line => width / Math.max(units(line), 1)));
}
const logo = (x, y, size) => `<image x="${x}" y="${y}" width="${size}" height="${size}" href="${brandUri}"/>`;
const rect = (x, y, w, h, r, fill, extra = '') => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${r}" fill="${fill}" ${extra}/>`;
const lines = (x, y, width, color = '#C6D4EA') => [0, 1, 2].map((n) => rect(x, y + n * 27, width - n * 24, 9, 4.5, color)).join('');
const tag = (x, y, title, fill = '#E7EEFF', ink = '#2057FF') => rect(x, y, Math.max(80, title.length * 16 + 34), 48, 14, fill) + text(x + 17, y + 33, title, 23, ink, 600);
function sheet(x, y, width, height, tint = '#FFFFFF', rotation = 0, label = '') {
  return `<g transform="translate(${x} ${y}) rotate(${rotation})" filter="url(#shadow)">${rect(0, 0, width, height, 26, tint, 'stroke="#DCE5F3" stroke-width="2"')}${rect(28, 36, 54, 54, 15, '#E5EEFF')}${lines(30, 122, width - 66)}${label ? tag(30, height - 78, label) : ''}</g>`;
}
function archive(x, y, size, rotation = 0, dark = false) {
  return `<g transform="translate(${x} ${y}) rotate(${rotation})" filter="url(#shadow)">${rect(0, 0, size, size * 1.12, 48, dark ? 'url(#night)' : 'url(#paper)', 'stroke="#CBD9EE" stroke-width="2"')}${rect(size * .12, size * .12, size * .76, size * .06, size * .03, dark ? '#475D7F' : '#D5E2F6')}${logo(size * .15, size * .26, size * .7)}</g>`;
}
function scene(index) {
  if (index === 0) return `<g transform="translate(1010 180)" filter="url(#shadow)">${rect(0,0,760,680,28,'#FFFFFF','stroke="#DCE5F3" stroke-width="2"')}${rect(0,0,760,65,28,'#F7F8FA')}${rect(0,33,760,32,0,'#F7F8FA')}${logo(24,16,34)}${rect(82,28,78,9,4.5,'#B8C6DC')}${rect(216,22,138,22,11,'#FFFFFF','stroke="#DCE5F3"')}${logo(332,116,96)}${rect(230,237,300,15,7.5,'#C2D0E5')}${rect(277,266,206,9,4.5,'#DAE4F1')}${rect(91,326,275,112,18,'#2057FF')}${rect(394,326,275,112,18,'#F8FAFE','stroke="#DCE5F3" stroke-width="2"')}${rect(120,353,52,52,13,'#FFFFFF','opacity=".25"')}${rect(423,353,52,52,13,'#E5EEFF')}${rect(188,354,142,10,5,'#FFFFFF','opacity=".9"')}${rect(491,354,142,10,5,'#9AAEC9')}${rect(188,377,113,8,4,'#FFFFFF','opacity=".55"')}${rect(491,377,113,8,4,'#D0DCEC')}${rect(91,468,578,108,18,'#F6F9FF','stroke="#CBD9EE" stroke-width="2" stroke-dasharray="8 8"')}${rect(265,511,230,10,5,'#C8D7EB')}${rect(0,624,760,56,0,'#F8FAFD')}</g>`;
  if (index === 1) return `${sheet(1020, 170, 285, 330, '#FFFFFF', -14, 'TAR')}${sheet(1470, 207, 265, 305, '#F9FCFF', 14, '7z')}<path d="M1175 420Q1320 540 1415 446" fill="none" stroke="#9BB8FF" stroke-width="8" stroke-linecap="round"/>${archive(1185, 390, 390, -4)}<g transform="translate(1570 650)" filter="url(#shadow)"><path d="M24 60V34a42 42 0 0 1 84 0v26" fill="none" stroke="#FAFCFF" stroke-width="18"/>${rect(0, 54, 132, 106, 28, '#2057FF')}<circle cx="66" cy="101" r="11" fill="white"/><path d="M66 110v20" stroke="white" stroke-width="8" stroke-linecap="round"/></g>`;
  if (index === 2) return `<g transform="translate(1085 235) rotate(-8)" filter="url(#shadow)">${rect(0, 0, 610, 430, 34, 'white')}${rect(20, 20, 570, 346, 22, 'url(#sky)')}<circle cx="420" cy="112" r="58" fill="#FFF7DC"/><path d="M20 275Q142 140 285 254T590 215V366H20" fill="#6EAFC3"/><path d="M20 322Q250 225 590 318V366H20" fill="#F7E9D0"/>${rect(28, 389, 190, 8, 4, '#D0DCF0')}</g>${sheet(1455, 520, 280, 305, '#FFFFFF', 9, 'TXT')}<circle cx="1127" cy="722" r="88" fill="#2057FF" filter="url(#shadow)"/><path d="M1078 722q49-66 98 0-49 66-98 0" fill="none" stroke="white" stroke-width="8"/><circle cx="1127" cy="722" r="14" fill="white"/>`;
  return `${archive(1005, 218, 330, -12)}${archive(1430, 247, 330, 11, true)}${archive(1195, 405, 355, -1)}<circle cx="1040" cy="787" r="40" fill="#FFF" stroke="#D8E3F6" stroke-width="3"/><path d="M1040 728v-14m0 146v-14m-59-59h-14m146 0h-14m-101-42-10-10m103 103-10-10m-83 0-10 10m103-103-10 10" stroke="#A5BADF" stroke-width="6" stroke-linecap="round"/><circle cx="1740" cy="735" r="47" fill="#1B3153"/><circle cx="1759" cy="715" r="42" fill="${v.background}"/>`;
}
function promo(locale, index, width, height) {
  activeFont = locale.font_family || v.font;
  const item = locale.promotions[index];
  let body = rect(0, 0, 1920, 1080, 0, v.background) + '<ellipse cx="1450" cy="430" rx="740" ry="620" fill="url(#glow)"/>';
  body += '<path d="M930 0v1080M0 922h1920" stroke="#E5EDFA" stroke-width="2"/>';
  body += logo(112, 63, 58) + text(186, 108, 'xdzip', 37, v.ink, 650) + text(112, 215, item.eyebrow, 23, v.blue, 600);
  const titleSize = index === 0 ? headlineSize(item.headline, v.headline_size, 760) : v.headline_size;
  item.headline.forEach((line, n) => body += text(108, 346 + n * 115, line, titleSize, v.ink, 650, 'start', 760));
  const base = item.headline.length === 3 ? 655 : 578;
  wrapped(item.body, 760 / v.body_size).forEach((line, n) => body += text(113, base + n * 49, line, v.body_size, v.muted, 400, 'start', 760));
  body += scene(index) + rect(1492, 70, 310, 50, 25, '#E6EDFD') + text(1647, 103, locale.illustration_label, 22, '#536C94', 500, 'middle');
  return svg(width, height, body);
}
function screenshotPromo(locale, index, capture, width, height, layoutEngine = imageTools) {
  activeFont = locale.font_family || v.font;
  const item = locale.promotions[index];
  const layout = spec.screenshot_artwork;
  const box = layoutEngine.placement(spec,capture,width,height);
  const ratio = width / 1920;
  let body = rect(0, 0, 1920, 1080, 0, v.background);
  body += '<ellipse cx="1440" cy="430" rx="820" ry="670" fill="url(#glow)"/>';
  body += '<path d="M-80 1050Q720 590 2030 1090M-80 1105Q720 645 2030 1145" fill="none" stroke="#D9E7FE" stroke-width="3"/>';
  body += logo(76, 66, 62) + text(154, 111, 'xdzip', 40, v.ink, 650);
  body += text(78, 235, item.eyebrow, 22, v.blue, 600, 'start', 490);
  const titleSize = index === 0 ? headlineSize(item.headline, layout.headline_size, 490) : layout.headline_size;
  item.headline.forEach((line, n) => body += text(74, 345 + n * 86, line, titleSize, v.ink, 650, 'start', 490));
  const base = item.headline.length === 3 ? 667 : 594;
  wrapped(item.body, 490 / layout.body_size).forEach((line, n) => body += text(78, base + n * 41, line, layout.body_size, v.muted, 400, 'start', 490));
  body += `<g transform="translate(82 792) rotate(-7)">${logo(0, 0, 120)}</g>`;
  // The screenshot supplies its own border; this background adds only shadow.
  body += `<defs><filter id="capture-shadow" x="-25%" y="-20%" width="150%" height="160%"><feDropShadow dx="0" dy="${layout.shadow_offset_y}" stdDeviation="${layout.shadow_blur}" flood-color="#173763" flood-opacity=".18"/></filter></defs>`;
  body += rect(box.imageX/ratio,box.imageY/ratio,box.imageWidth/ratio,box.imageHeight/ratio,
    capture.corner_radius*box.scale/ratio,'#FFFFFF','filter="url(#capture-shadow)"');
  return svg(width, height, body);
}
function poster(locale, width, height) {
  activeFont = locale.font_family || v.font;
  let body = rect(0, 0, 720, 1080, 0, 'url(#ocean)') + '<circle cx="675" cy="125" r="350" fill="#66D7F2" opacity=".12"/><path d="M-60 850Q350 600 800 935M-60 918Q350 668 800 1003M-60 986Q350 736 800 1071" fill="none" stroke="#B1D9FF" stroke-opacity=".16" stroke-width="2"/>';
  body += text(64, 111, 'xdzip', 66, '#FFFFFF', 650);
  locale.poster_headline.forEach((line, n) => body += text(64, 202 + n * 65, line, 51, '#FFFFFF', 600, 'start', 590));
  body += logo(155, 306, 410);
  return svg(width, height, body, '0 0 720 1080');
}
function square(locale, width, height) {
  activeFont = locale.font_family || v.font;
  const body = rect(0, 0, 1080, 1080, 0, 'url(#ocean)') + '<circle cx="1000" cy="0" r="660" fill="#7DE4FF" opacity=".13"/><path d="M-100 1030Q550 610 1220 1010M-100 1090Q550 670 1220 1070" fill="none" stroke="#B1D9FF" stroke-opacity=".16" stroke-width="2"/>' + logo(310, 58, 460) + text(540, 618, 'xdzip', 104, '#FFFFFF', 650, 'middle') + text(540, 693, locale.square_tagline, 34, '#E4EEFF', 400, 'middle', 930);
  return svg(width, height, body, '0 0 1080 1080');
}
function hero(width, height) {
  let body = rect(0, 0, 1920, 1080, 0, 'url(#ocean)') + '<circle cx="1680" cy="40" r="790" fill="#82E7FF" opacity=".12"/><path d="M-200 1070Q950 490 2150 1110M-200 1130Q950 550 2150 1170M-200 1190Q950 610 2150 1230" fill="none" stroke="#B7E6FF" stroke-opacity=".18" stroke-width="3"/>';
  body += sheet(410, 267, 380, 460, '#EDF6FF', -17) + sheet(1285, 157, 340, 425, '#E9F8FF', 17);
  body += logo(654, 87, 612);
  body += '<ellipse cx="967" cy="850" rx="490" ry="45" fill="#031E75" opacity=".16"/>';
  return svg(width, height, body);
}
function icon(size) {
  return brand.replace(/width="1024" height="1024"/, `width="${size}" height="${size}"`);
}
const en = 'developers/market/windows';
const zh = 'developers/zh-CN/market/windows';
const localeBase = key => key === 'en-US' ? en : key === 'zh-CN' ? zh : `${en}/locales/${key}`;
for (const size of [71, 150, 300, 512, 1024]) jobs.push({file:`${en}/images/branding/${size <= 300 ? 'app-icon' : 'avatar'}-${size}.png`, width:size, height:size, role:size <= 300 ? 'app-tile-icon' : 'reusable-avatar', locale:'shared', source:icon(size), transparent:true});
for (const [w,h] of largeSizes) jobs.push({file:`${en}/images/branding/hero-${w}x${h}.png`, width:w,height:h,role:'text-free-hero',locale:'shared',source:hero(w,h)});
for (const [key, locale] of Object.entries(spec.locales)) {
  const base = localeBase(key);
  for (const [w,h] of [[720,1080],[1440,2160]]) jobs.push({file:`${base}/images/branding/poster-${w}x${h}.png`,width:w,height:h,role:'poster',locale:key,source:poster(locale,w,h)});
  for (const size of [1080,2160]) jobs.push({file:`${base}/images/branding/box-art-${size}.png`,width:size,height:size,role:'box-art',locale:key,source:square(locale,size,size)});
  for (let i=0;i<locale.promotions.length;i++) for (const [w,h] of largeSizes) jobs.push({file:`${base}/images/promotional/${locale.promotions[i].id}-${w}x${h}.png`,width:w,height:h,role:'feature-illustration',locale:key,source:promo(locale,i,w,h)});
}
if (captures) {
  for (const capture of captures.images) {
    const bytes = fs.readFileSync(path.join(ROOT, capture.file));
    if (hash(bytes) !== capture.sha256) throw Error(`Changed screenshot source ${capture.file}`);
    const locale = spec.locales[capture.locale];
    const index = locale.promotions.findIndex(p => p.id === capture.scene);
    if (index < 0) throw Error(`Unknown screenshot scene ${capture.scene}`);
    jobs.push({file:capture.file, width:capture.width, height:capture.height, role:'app-screenshot', locale:capture.locale, source:bytes, input:true,transparent:true,capture});
    const nativeBytes = fs.readFileSync(path.join(ROOT,capture.native_file));
    if (hash(nativeBytes) !== capture.source_sha256) throw Error(`Changed native source ${capture.native_file}`);
    jobs.push({file:capture.native_file,width:capture.native_width,height:capture.native_height,role:'native-screenshot-source',locale:capture.locale,source:nativeBytes,input:true,capture});
    const cutoutBytes = fs.readFileSync(path.join(ROOT,capture.cutout_file));
    if (hash(cutoutBytes) !== capture.cutout_sha256) throw Error(`Changed screenshot cutout ${capture.cutout_file}`);
    const base = localeBase(capture.locale);
    const [minimumWidth,minimumHeight] = spec.screenshot_artwork.store_canvas;
    const canvas = {width:minimumWidth,height:minimumHeight,background:spec.screenshot_artwork.store_background};
    if (capture.cutout_width > canvas.width || capture.cutout_height > canvas.height) throw Error('Native screenshot exceeds Store canvas');
    const left = Math.floor((canvas.width - capture.cutout_width) / 2);
    const top = Math.floor((canvas.height - capture.cutout_height) / 2);
    jobs.push({file:`${base}/images/store-screenshots/${capture.scene}.png`, width:canvas.width, height:canvas.height,
      role:'store-screenshot', locale:capture.locale, source:JSON.stringify({capture_sha256:capture.cutout_sha256,canvas,left,top}),
      canvas, left, top, capture});
    for (const [width,height] of largeSizes) {
      const background = screenshotPromo(locale,index,capture,width,height);
      const box = imageTools.placement(spec,capture,width,height);
      jobs.push({file:`${base}/images/combined/${capture.scene}-${width}x${height}.png`,width,height,role:'screenshot-artwork',locale:capture.locale,capture,
        source:JSON.stringify({background,box,capture_sha256:capture.cutout_sha256}),background,box});
    }
  }
}
function validateCopy() {
  const productLanguages = JSON.parse(fs.readFileSync(path.join(repository,'resources/ui-languages.json'),'utf8')).languages.map(item => item.id).sort();
  if (JSON.stringify(Object.keys(spec.locales).sort()) !== JSON.stringify(productLanguages)) throw Error('Store languages differ from product languages');
  const a=spec.locales['en-US'],b=spec.locales['zh-CN'];
  if(a.features.length!==b.features.length || a.promotions.length!==4 || JSON.stringify(a.promotions.map(p=>p.id))!==JSON.stringify(b.promotions.map(p=>p.id))) throw Error('Feature or artwork inventory differs');
  for (const [key, l] of Object.entries(spec.locales)) {
    if (l.features.length !== a.features.length || JSON.stringify(l.promotions.map(p=>p.id)) !== JSON.stringify(a.promotions.map(p=>p.id))) throw Error(`${key} artwork inventory differs`);
    const limits = {product_name:50,short_title:50,sort_title:255,voice_title:255,developer:255,short_description:270,description:10000,release_notes:1500,copyright:200,additional_license_terms:10000};
    for (const [name,max] of Object.entries(limits)) if (l[name].length > max) throw Error(`${key} ${name} exceeds ${max}`);
    if (!l.description.trim() || !l.short_description.trim() || l.features.some(value => !value.trim()) || l.keywords.some(value => !value.trim())) throw Error(`${key} copy is incomplete`);
    if (l.promotions.some(p => !p.eyebrow || !p.headline.length || p.headline.length > 3 || !p.body.length || !p.caption || wrapped(p.body, 490 / spec.screenshot_artwork.body_size).length > 4 || wrapped(p.body, 760 / v.body_size).length > 4)) throw Error(`${key} promotional copy is incomplete or too long`);
    if (l.features.length > 20 || l.features.some(s=>s.length>200)) throw Error(`${key} features exceed portal limits`);
    if (l.keywords.length > 7 || l.keywords.some(s=>s.length>40) || l.keywords.join(' ').trim().split(/\s+/).length>21) throw Error(`${key} keywords exceed portal limits`);
    if (l.promotions.some(p=>p.caption.length>200)) throw Error(`${key} caption exceeds 200`);
    if (Object.keys(l.screenshot_captions).length !== l.promotions.length ||
        l.promotions.some(({id})=>!l.screenshot_captions[id] || l.screenshot_captions[id].length>200)) {
      throw Error(`${key} screenshot captions are incomplete or exceed 200 characters`);
    }
    if (/https?:|<\/?\w/.test(l.description)) throw Error(`${key} description is not plain text`);
    if (l.additional_license_terms || l.voice_title || l.sort_title) throw Error('Optional empty fields must stay intentional');
  }
  if (jobs.filter(j=>['feature-illustration','screenshot-artwork'].includes(j.role)).some(j=>String(j.source).includes('Windows 11, x64'))) throw Error('Promotional footer system label must stay removed');
  if (jobs.filter(j=>j.role==='text-free-hero').some(j=>/<text\b/.test(j.source))) throw Error('Hero artwork must not contain text');
  if (captures) {
    if (captures.baseline_revision !== spec.baseline.revision || captures.package_sha256 !== spec.baseline.package_sha256 || captures.runtime_exe_sha256 !== spec.baseline.runtime_exe_sha256) throw Error('Screenshot candidate differs from listing baseline');
    const capturedLanguages = [...new Set(captures.images.map(c => c.locale))];
    if (!capturedLanguages.length || capturedLanguages.some(key => !spec.locales[key])) throw Error('Unknown capture language');
    const expected = capturedLanguages.flatMap(key => spec.locales[key].promotions.map(p => `${key}/${p.id}`)).sort();
    const actual = captures.images.map(c => `${c.locale}/${c.scene}`).sort();
    if (JSON.stringify(expected) !== JSON.stringify(actual)) throw Error('Screenshot language or scene inventory differs');
    if (captures.schema_version !== 3 || captures.window_mode !== 'default') throw Error('Cleaned screenshots with native source inventory required');
    if (captures.images.some(c => !c.caption || c.caption.length > 200 || c.caption !== spec.locales[c.locale].screenshot_captions[c.scene])) throw Error('Screenshot captions differ or exceed portal limits');
  }
}
function listingMarkdown(key,l) {
  const chinese=key==='zh-CN';
  const h=chinese?['商店一览文案，中文（中国）','适用基线','产品名称','说明','此版本的新增功能','首次提交，留空。不要把这句说明粘贴到字段中。','产品功能','逐项填写，每行是一个独立功能，不要复制列表编号。','补充字段','字段','填写内容','短标题','语音标题','简短描述','其他信息','关键字','逐项添加，共七项。','版权和商标信息','其他许可条款','留空。项目未定义额外的商店专用许可条款；应用中的开源许可和第三方通知仍然适用。','开发者','支持与隐私链接','网站','支持','隐私政策','项目许可','系统范围','宣传图说明文字','这些是功能示意图，不是实机截图。图片文件和字幕不可作为“已满足必需截图栏”的保证。','留空，桌面应用不使用 Xbox 语音标题。']:['Store listing copy, English (US)','Baseline','Product name','Description',"What's new in this version",'First submission: leave empty. Do not paste this instruction into the field.','Product features','Enter each feature separately, without the list numbers.','Supplemental fields','Field','Value','Short title','Voice title','Short description','Other information','Keywords','Add separately, seven items in total.','Copyright and trademark information','Additional license terms','Leave empty. The project defines no additional Store-specific terms; the open-source licenses and third-party notices shipped with the app still apply.','Developer','Website, support and privacy','Website','Support','Privacy policy','Project license','System scope','Promotional image captions','These images are feature illustrations, not app screenshots. Their files and captions do not establish that the required screenshot field is satisfied.','Leave empty; this desktop app does not use an Xbox voice title.'];
  const target = chinese
    ? '用于 Windows 桌面商店一览。选择与本次提交相符的新增功能文案，截图来源见[材料说明](README.md)。'
    : 'Use this copy for the Windows desktop listing. Choose release notes for the current submission and see the [materials guide](README.md) for screenshot provenance.';
  const changes = l.release_notes ? ['```text', l.release_notes, '```'].join('\n') : h[5];
  let s=`# ${h[0]}\n\n${chinese?'[English](../../../market/windows/listing.md) | 简体中文':'English | [简体中文](../../zh-CN/market/windows/listing.md)'}\n\n## ${h[1]}\n\n${target}\n\n## ${h[2]}\n\n\`\`\`text\n${l.product_name}\n\`\`\`\n\n## ${h[3]}\n\n\`\`\`text\n${l.description}\n\`\`\`\n\n## ${h[4]}\n\n${changes}\n\n## ${h[6]}\n\n${h[7]}\n\n`;
  l.features.forEach((f,i)=>s+=`${i+1}. ${f}\n`);
  s+=`\n## ${h[8]}\n\n| ${h[9]} | ${h[10]} |\n| --- | --- |\n| ${h[11]} | ${l.short_title} |\n| ${h[12]} | ${h[29]} |\n\n### ${h[13]}\n\n\`\`\`text\n${l.short_description}\n\`\`\`\n\n## ${h[14]}\n\n### ${h[15]}\n\n${h[16]}\n\n`;
  s = s.replace(`| ${h[12]} |`, `${chinese?'| 排序标题 | 留空，名称没有其他排序形式。 |':'| Sort title | Leave empty; no alternate sorting form is needed. |'}\n| ${h[12]} |`);
  l.keywords.forEach(k=>s+=`- ${k}\n`);
  s+=`\n### ${h[17]}\n\n\`\`\`text\n${l.copyright}\n\`\`\`\n\n### ${h[18]}\n\n${h[19]}\n\n### ${h[20]}\n\n\`\`\`text\n${l.developer}\n\`\`\`\n\n## ${h[21]}\n\n| ${h[9]} | URL |\n| --- | --- |\n| ${h[22]} | ${spec.links.website} |\n| ${h[23]} | ${spec.links.support} |\n| ${h[24]} | ${l.privacy_url} |\n| ${h[25]} | ${spec.links.license} |\n\n## ${h[26]}\n\n${l.system_requirements.join('\n')}\n\n## ${h[27]}\n\n${h[28]}\n\n`;
  l.promotions.forEach(p=>s+=`### ${p.id}\n\n\`\`\`text\n${p.caption}\n\`\`\`\n\n`);
  s += `## ${chinese?'应用截图与融合宣传图':'App screenshots and combined artwork'}\n\n${chinese?'桌面截图栏上传本地导出目录中 promotional/ 内的 1920 × 1080 融合宣传图。':'Upload the 1920 × 1080 combined artwork in promotional/ from the local export to the desktop screenshot field.'}\n\n`;
  for (const p of l.promotions) {
    s += `### ${p.id}\n\n\`\`\`text\n${l.screenshot_captions[p.id]}\n\`\`\`\n\n`;
  }
  return s.trimEnd()+'\n';
}
async function main() {
  validateCopy(); const entries=[];
  if (listingsOnly) {
    for(const key of ['en-US','zh-CN']) {
      const l = spec.locales[key];
      const destination=path.join(ROOT,key==='en-US'?en:zh,'listing.md');
      const content=listingMarkdown(key,l);
      if(check) {
        if(fs.readFileSync(destination,'utf8').replaceAll('\r\n','\n')!==content) throw Error(`Stale listing copy ${key}`);
      } else fs.writeFileSync(destination,content);
    }
    console.log('Ten-language Store copy, bilingual listing documents and field limits passed.');
    return;
  }
  for (const job of jobs) {
    const destination=path.join(ROOT,job.file); const source_sha256=hash(job.source);
    const old=previous?.assets.find(a=>a.file===job.file);
    const unchanged = old && old.source_sha256 === source_sha256 && fs.existsSync(destination) && hash(fs.readFileSync(destination)) === old.sha256;
    let background = null,layer = null;
    if (job.canvas) {
      background = await sharp({create:{...job.canvas,channels:3}}).png().toBuffer();
      layer = fs.readFileSync(path.join(ROOT,job.capture.cutout_file));
    } else if (job.background) {
      background = await sharp(Buffer.from(job.background)).png().toBuffer();
      layer = await imageTools.screenshotLayer(fs.readFileSync(path.join(ROOT,job.capture.cutout_file)),job.box);
    }
    if (!check && !job.input && !unchanged) {
      fs.mkdirSync(path.dirname(destination),{recursive:true});
      if (background) fs.writeFileSync(destination,await imageTools.composite(background,layer,job.canvas?job.left:job.box.imageX,job.canvas?job.top:job.box.imageY));
      else await sharp(Buffer.from(job.source)).png({compressionLevel:9}).toFile(destination);
    }
    const bytes=fs.readFileSync(destination); const meta=await sharp(bytes).metadata();
    if (meta.format!=='png' || meta.width!==job.width || meta.height!==job.height || bytes.length>5000000) throw Error(`Invalid image ${job.file}`);
    if (!job.transparent && !(await sharp(bytes).stats()).isOpaque) throw Error(`Unexpected transparency ${job.file}`);
    if (job.input) {
      const capture = job.capture;
      if (job.role === 'native-screenshot-source') {
        if (hash(await sharp(bytes).removeAlpha().raw().toBuffer()) !== capture.pixels_sha256) throw Error(`Changed native UI pixels ${job.file}`);
      } else {
        const expected = await imageTools.makeCutout(fs.readFileSync(path.join(ROOT,capture.native_file)),capture.native_width,capture.native_height,capture.dpi,spec.screenshot_artwork.cleanup);
        if (!expected.png.equals(bytes)) throw Error(`Changed native cutout pixels or alpha ${job.file}`);
      }
    }
    if (job.canvas) {
      if (meta.width < 1366 || meta.height < 768) throw Error(`Store screenshot canvas is too small ${job.file}`);
      await imageTools.assertOverlay(bytes,layer,job.left,job.top,background);
    }
    if (job.role === 'screenshot-artwork') {
      if (job.box.scale > 1 || (job.width === 3840 && job.box.scale !== 1)) throw Error(`Native screenshot scale differs ${job.file}`);
      await imageTools.assertOverlay(bytes,layer,job.box.imageX,job.box.imageY,background);
    }
    const entry={file:job.file,width:job.width,height:job.height,role:job.role,locale:job.locale,source_sha256,sha256:hash(bytes)};
    if (check) {
      if (!old || JSON.stringify(old)!==JSON.stringify(entry)) throw Error(`Stale or changed asset ${job.file}`);
    }
    entries.push(entry);
  }
  if (check && previous.assets.length!==jobs.length) throw Error('Unexpected asset inventory entries');
  if (!check) fs.writeFileSync(indexFile,JSON.stringify({schema_version:1,brand_source:'design/assets/brand/xdzip.svg',assets:entries},null,2)+'\n');
  console.log(`${check?'Validated':'Rendered'} ${jobs.length} PNG assets; field limits and source hashes passed.`);
}
module.exports = {screenshotPromo, promo, poster, square, hero, icon};
if (require.main === module) {
  const standalone = process.argv.includes('--backgrounds-only');
  const work = standalone ? require('./build-platform-store-assets.cjs').build('windows', process.argv.slice(2).filter(arg => arg !== '--backgrounds-only')) : main();
  work.catch(e=>{console.error(e.message);process.exitCode=1;});
}
