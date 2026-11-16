#!/usr/bin/env node
/* Render editable vector marketing artwork without launching a desktop or browser. */
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const sharp = require('sharp');
const ROOT = path.resolve(__dirname, '..');
const spec = JSON.parse(fs.readFileSync(path.join(ROOT, 'specs/marketing/windows-store.json'), 'utf8'));
const brand = fs.readFileSync(path.join(ROOT, 'design/assets/brand/xdzip.svg'), 'utf8');
const brandUri = 'data:image/svg+xml;base64,' + Buffer.from(brand).toString('base64');
const check = process.argv.includes('--check');
const indexFile = path.join(ROOT, 'specs/marketing/windows-store-assets.json');
const captureFile = path.join(ROOT, 'specs/marketing/windows-store-captures.json');
const captures = fs.existsSync(captureFile) ? JSON.parse(fs.readFileSync(captureFile, 'utf8')) : null;
const previous = check ? JSON.parse(fs.readFileSync(indexFile, 'utf8')) : null;
const jobs = [];
const hash = data => crypto.createHash('sha256').update(data).digest('hex');
const esc = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');
const v = spec.visual;
const svg = (width, height, body, view = '0 0 1920 1080') => `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="${view}"><defs><linearGradient id="paper" x2=".9" y2="1"><stop stop-color="#FFFFFF"/><stop offset="1" stop-color="#E8F0FF"/></linearGradient><linearGradient id="ocean" x2=".85" y2="1"><stop stop-color="#145DE5"/><stop offset=".48" stop-color="#2057FF"/><stop offset="1" stop-color="#142C95"/></linearGradient><linearGradient id="night" x2="1" y2="1"><stop stop-color="#213756"/><stop offset="1" stop-color="#0A1530"/></linearGradient><linearGradient id="sky" x2="0" y2="1"><stop stop-color="#73B7FF"/><stop offset="1" stop-color="#C6ECF4"/></linearGradient><radialGradient id="glow"><stop stop-color="#BBDEFF" stop-opacity=".8"/><stop offset="1" stop-color="#BBDEFF" stop-opacity="0"/></radialGradient><filter id="shadow" x="-40%" y="-40%" width="180%" height="200%"><feDropShadow dx="0" dy="26" stdDeviation="26" flood-color="#203E7B" flood-opacity=".17"/></filter></defs>${body}</svg>`;
const text = (x, y, content, size = 32, color = v.ink, weight = 400, anchor = 'start') => `<text x="${x}" y="${y}" font-family="${v.font}" font-size="${size}" font-weight="${weight}" fill="${color}" text-anchor="${anchor}">${esc(content)}</text>`;
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
  if (index === 0) return `${sheet(1225, 166, 290, 340, '#F6FAFF', 12, 'TXT')}${archive(1050, 288, 410, -10)}${sheet(1435, 418, 265, 320, '#FFFFFF', 12, 'PNG')}<path d="M990 430Q947 488 980 550" fill="none" stroke="#63CFE5" stroke-width="8" stroke-linecap="round"/><circle cx="979" cy="552" r="13" fill="#2057FF"/>${tag(1140, 828, 'ZIP')}${tag(1250, 828, '7z')}${tag(1360, 828, 'RAR')}${tag(1485, 828, 'TAR')}`;
  if (index === 1) return `${sheet(1020, 170, 285, 330, '#FFFFFF', -14, 'TAR')}${sheet(1470, 207, 265, 305, '#F9FCFF', 14, '7z')}<path d="M1175 420Q1320 540 1415 446" fill="none" stroke="#9BB8FF" stroke-width="8" stroke-linecap="round"/>${archive(1185, 390, 390, -4)}<g transform="translate(1570 650)" filter="url(#shadow)"><path d="M24 60V34a42 42 0 0 1 84 0v26" fill="none" stroke="#FAFCFF" stroke-width="18"/>${rect(0, 54, 132, 106, 28, '#2057FF')}<circle cx="66" cy="101" r="11" fill="white"/><path d="M66 110v20" stroke="white" stroke-width="8" stroke-linecap="round"/></g>`;
  if (index === 2) return `<g transform="translate(1085 235) rotate(-8)" filter="url(#shadow)">${rect(0, 0, 610, 430, 34, 'white')}${rect(20, 20, 570, 346, 22, 'url(#sky)')}<circle cx="420" cy="112" r="58" fill="#FFF7DC"/><path d="M20 275Q142 140 285 254T590 215V366H20" fill="#6EAFC3"/><path d="M20 322Q250 225 590 318V366H20" fill="#F7E9D0"/>${rect(28, 389, 190, 8, 4, '#D0DCF0')}</g>${sheet(1455, 520, 280, 305, '#FFFFFF', 9, 'TXT')}<circle cx="1127" cy="722" r="88" fill="#2057FF" filter="url(#shadow)"/><path d="M1078 722q49-66 98 0-49 66-98 0" fill="none" stroke="white" stroke-width="8"/><circle cx="1127" cy="722" r="14" fill="white"/>`;
  return `${archive(1005, 218, 330, -12)}${archive(1430, 247, 330, 11, true)}${archive(1195, 405, 355, -1)}<circle cx="1040" cy="787" r="40" fill="#FFF" stroke="#D8E3F6" stroke-width="3"/><path d="M1040 728v-14m0 146v-14m-59-59h-14m146 0h-14m-101-42-10-10m103 103-10-10m-83 0-10 10m103-103-10 10" stroke="#A5BADF" stroke-width="6" stroke-linecap="round"/><circle cx="1740" cy="735" r="47" fill="#1B3153"/><circle cx="1759" cy="715" r="42" fill="${v.background}"/>`;
}
function promo(locale, index, width, height) {
  const item = locale.promotions[index];
  let body = rect(0, 0, 1920, 1080, 0, v.background) + '<ellipse cx="1450" cy="430" rx="740" ry="620" fill="url(#glow)"/>';
  body += '<path d="M930 0v1080M0 922h1920" stroke="#E5EDFA" stroke-width="2"/>';
  body += logo(112, 63, 58) + text(186, 108, 'xdzip', 37, v.ink, 650) + text(112, 215, item.eyebrow, 23, v.blue, 600);
  item.headline.forEach((line, n) => body += text(108, 346 + n * 115, line, v.headline_size, v.ink, 650));
  const base = item.headline.length === 3 ? 655 : 578;
  item.body.forEach((line, n) => body += text(113, base + n * 49, line, v.body_size, v.muted));
  body += scene(index) + rect(1492, 70, 310, 50, 25, '#E6EDFD') + text(1647, 103, locale.illustration_label, 22, '#536C94', 500, 'middle');
  body += text(113, 1009, 'Windows 11, x64', 22, '#64789A');
  return svg(width, height, body);
}
function screenshotPlacement(capture) {
  const layout = spec.screenshot_artwork;
  const [frameX, frameY, frameWidth, frameHeight] = layout.frame;
  const scale = Math.min(layout.max_capture_scale, frameWidth / capture.width, frameHeight / capture.height);
  const imageWidth = capture.width * scale;
  const imageHeight = capture.height * scale;
  return {scale, imageWidth, imageHeight,
    imageX:Math.round(frameX + (frameWidth - imageWidth) / 2),
    imageY:Math.round(frameY + (frameHeight - imageHeight) / 2)};
}
function screenshotPromo(locale, index, capture, width, height) {
  const item = locale.promotions[index];
  const layout = spec.screenshot_artwork;
  const {imageWidth,imageHeight,imageX,imageY} = screenshotPlacement(capture);
  const uri = 'data:image/png;base64,' + capture.bytes.toString('base64');
  let body = rect(0, 0, 1920, 1080, 0, v.background);
  body += '<ellipse cx="1440" cy="430" rx="820" ry="670" fill="url(#glow)"/>';
  body += '<path d="M-80 1050Q720 590 2030 1090M-80 1105Q720 645 2030 1145" fill="none" stroke="#D9E7FE" stroke-width="3"/>';
  body += logo(76, 66, 62) + text(154, 111, 'xdzip', 40, v.ink, 650);
  body += text(78, 235, item.eyebrow, 22, v.blue, 600);
  item.headline.forEach((line, n) => body += text(74, 345 + n * 86, line, layout.headline_size, v.ink, 650));
  const base = item.headline.length === 3 ? 667 : 594;
  item.body.forEach((line, n) => body += text(78, base + n * 41, line, layout.body_size, v.muted));
  body += `<g transform="translate(82 792) rotate(-7)">${logo(0, 0, 120)}</g>`;
  body += text(78, 1010, 'Windows 11, x64', 22, '#64789A');
  body += rect(1318, 75, 526, 50, 25, '#E6EDFD');
  body += text(1581, 108, locale.screenshot_artwork_label, 22, '#536C94', 500, 'middle');
  body += rect(imageX - 10, imageY - 10, imageWidth + 20, imageHeight + 20, 16, '#FFFFFF', 'filter="url(#shadow)"');
  body += `<image x="${imageX}" y="${imageY}" width="${imageWidth}" height="${imageHeight}" href="${uri}"/>`;
  return svg(width, height, body);
}
function poster(locale, width, height) {
  let body = rect(0, 0, 720, 1080, 0, 'url(#ocean)') + '<circle cx="675" cy="125" r="350" fill="#66D7F2" opacity=".12"/><path d="M-60 850Q350 600 800 935M-60 918Q350 668 800 1003M-60 986Q350 736 800 1071" fill="none" stroke="#B1D9FF" stroke-opacity=".16" stroke-width="2"/>';
  body += text(64, 111, 'xdzip', 66, '#FFFFFF', 650);
  locale.poster_headline.forEach((line, n) => body += text(64, 202 + n * 65, line, 51, '#FFFFFF', 600));
  body += logo(155, 306, 410);
  return svg(width, height, body, '0 0 720 1080');
}
function square(locale, width, height) {
  const body = rect(0, 0, 1080, 1080, 0, 'url(#ocean)') + '<circle cx="1000" cy="0" r="660" fill="#7DE4FF" opacity=".13"/><path d="M-100 1030Q550 610 1220 1010M-100 1090Q550 670 1220 1070" fill="none" stroke="#B1D9FF" stroke-opacity=".16" stroke-width="2"/>' + logo(310, 58, 460) + text(540, 618, 'xdzip', 104, '#FFFFFF', 650, 'middle') + text(540, 693, locale.square_tagline, 34, '#E4EEFF', 400, 'middle');
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
const en = 'docs/market/windows';
const zh = 'docs/zh-CN/market/windows';
for (const size of [71, 150, 300, 512, 1024]) jobs.push({file:`${en}/images/branding/${size <= 300 ? 'app-icon' : 'avatar'}-${size}.png`, width:size, height:size, role:size <= 300 ? 'app-tile-icon' : 'reusable-avatar', locale:'shared', source:icon(size), transparent:true});
for (const [w,h] of [[1920,1080],[3840,2160]]) jobs.push({file:`${en}/images/branding/hero-${w}x${h}.png`, width:w,height:h,role:'text-free-hero',locale:'shared',source:hero(w,h)});
for (const [key, locale] of Object.entries(spec.locales)) {
  const base = key === 'en-US' ? en : zh;
  for (const [w,h] of [[720,1080],[1440,2160]]) jobs.push({file:`${base}/images/branding/poster-${w}x${h}.png`,width:w,height:h,role:'poster',locale:key,source:poster(locale,w,h)});
  for (const size of [1080,2160]) jobs.push({file:`${base}/images/branding/box-art-${size}.png`,width:size,height:size,role:'box-art',locale:key,source:square(locale,size,size)});
  for (let i=0;i<locale.promotions.length;i++) for (const [w,h] of [[1920,1080],[3840,2160]]) jobs.push({file:`${base}/images/promotional/${locale.promotions[i].id}-${w}x${h}.png`,width:w,height:h,role:'feature-illustration',locale:key,source:promo(locale,i,w,h)});
}
if (captures) {
  for (const capture of captures.images) {
    const bytes = fs.readFileSync(path.join(ROOT, capture.file));
    if (hash(bytes) !== capture.sha256) throw Error(`Changed screenshot source ${capture.file}`);
    const locale = spec.locales[capture.locale];
    const index = locale.promotions.findIndex(p => p.id === capture.scene);
    if (index < 0) throw Error(`Unknown screenshot scene ${capture.scene}`);
    jobs.push({file:capture.file, width:capture.width, height:capture.height, role:'app-screenshot', locale:capture.locale, source:bytes, input:true});
    const base = capture.locale === 'en-US' ? en : zh;
    const [minimumWidth,minimumHeight] = spec.screenshot_artwork.store_canvas;
    const canvas = {width:Math.max(minimumWidth,capture.width), height:Math.max(minimumHeight,capture.height), background:spec.screenshot_artwork.store_background};
    const left = Math.floor((canvas.width - capture.width) / 2);
    const top = Math.floor((canvas.height - capture.height) / 2);
    jobs.push({file:`${base}/images/store-screenshots/${capture.scene}.png`, width:canvas.width, height:canvas.height,
      role:'store-screenshot', locale:capture.locale, source:JSON.stringify({capture_sha256:capture.sha256,canvas,left,top}),
      canvas, left, top, capture});
    for (const [width,height] of [[1920,1080],[3840,2160]]) {
      jobs.push({file:`${base}/images/combined/${capture.scene}-${width}x${height}.png`, width, height, role:'screenshot-artwork', locale:capture.locale, capture, source:screenshotPromo(locale,index,{...capture,bytes},width,height)});
    }
  }
}
function validateCopy() {
  if (Object.keys(spec.locales).sort().join(',')!=='en-US,zh-CN') throw Error('Expected the two listing locales');
  const a=spec.locales['en-US'],b=spec.locales['zh-CN'];
  if(a.features.length!==b.features.length || a.promotions.length!==4 || JSON.stringify(a.promotions.map(p=>p.id))!==JSON.stringify(b.promotions.map(p=>p.id))) throw Error('Bilingual feature or artwork inventory differs');
  for (const [key, l] of Object.entries(spec.locales)) {
    const limits = {product_name:50,short_title:50,voice_title:255,developer:255,short_description:270,description:10000,release_notes:1500,copyright:200,additional_license_terms:10000};
    for (const [name,max] of Object.entries(limits)) if (l[name].length > max) throw Error(`${key} ${name} exceeds ${max}`);
    if (l.features.length > 20 || l.features.some(s=>s.length>200)) throw Error(`${key} features exceed portal limits`);
    if (l.keywords.length > 7 || l.keywords.some(s=>s.length>40) || l.keywords.join(' ').trim().split(/\s+/).length>21) throw Error(`${key} keywords exceed portal limits`);
    if (l.promotions.some(p=>p.caption.length>200)) throw Error(`${key} caption exceeds 200`);
    if (/https?:|<\/?\w/.test(l.description)) throw Error(`${key} description is not plain text`);
    if (l.release_notes || l.additional_license_terms || l.voice_title) throw Error('First submission and desktop-only empty fields must stay intentional');
  }
  if (jobs.filter(j=>j.role==='text-free-hero').some(j=>/<text\b/.test(j.source))) throw Error('Hero artwork must not contain text');
  if (captures) {
    if (captures.baseline_revision !== spec.baseline.revision || captures.package_sha256 !== spec.baseline.package_sha256 || captures.runtime_exe_sha256 !== spec.baseline.runtime_exe_sha256) throw Error('Screenshot candidate differs from listing baseline');
    const expected = Object.entries(spec.locales).flatMap(([key,l]) => l.promotions.map(p => `${key}/${p.id}`)).sort();
    const actual = captures.images.map(c => `${c.locale}/${c.scene}`).sort();
    if (JSON.stringify(expected) !== JSON.stringify(actual)) throw Error('Screenshot language or scene inventory differs');
    if (captures.window_mode !== 'default' || captures.images.some(c => JSON.stringify([c.width,c.height]) !== JSON.stringify(spec.screenshot_artwork.default_capture_size))) throw Error('Screenshots must preserve the specified default window size');
    if (captures.images.some(c => !c.caption || c.caption.length > 200 || c.caption !== spec.locales[c.locale].screenshot_captions[c.scene])) throw Error('Screenshot captions differ or exceed portal limits');
    if (captures.images.some(c => JSON.stringify(c.cursor_cleanup_strip) !== JSON.stringify([0,c.height - spec.screenshot_artwork.cursor_cleanup_height,c.width,spec.screenshot_artwork.cursor_cleanup_height]))) throw Error('Screenshot cleanup region differs from its source specification');
  }
}
function listingMarkdown(key,l) {
  const chinese=key==='zh-CN';
  const h=chinese?['商店一览文案，中文（中国）','适用基线','产品名称','说明','此版本的新增功能','首次提交，留空。不要把这句说明粘贴到字段中。','产品功能','逐项填写，每行是一个独立功能，不要复制列表编号。','补充字段','字段','填写内容','短标题','语音标题','简短描述','其他信息','关键字','逐项添加，共七项。','版权和商标信息','其他许可条款','留空。项目未定义额外的商店专用许可条款；应用中的开源许可和第三方通知仍然适用。','开发者','支持与隐私链接','网站','支持','隐私政策','项目许可','系统范围','宣传图说明文字','这些是功能示意图，不是实机截图。图片文件和字幕不可作为“已满足必需截图栏”的保证。','留空，桌面应用不使用 Xbox 语音标题。']:['Store listing copy, English (US)','Baseline','Product name','Description',"What's new in this version",'First submission: leave empty. Do not paste this instruction into the field.','Product features','Enter each feature separately, without the list numbers.','Supplemental fields','Field','Value','Short title','Voice title','Short description','Other information','Keywords','Add separately, seven items in total.','Copyright and trademark information','Additional license terms','Leave empty. The project defines no additional Store-specific terms; the open-source licenses and third-party notices shipped with the app still apply.','Developer','Website, support and privacy','Website','Support','Privacy policy','Project license','System scope','Promotional image captions','These images are feature illustrations, not app screenshots. Their files and captions do not establish that the required screenshot field is satisfied.','Leave empty; this desktop app does not use an Xbox voice title.'];
  let s=`# ${h[0]}\n\n${chinese?'[English](../../../market/windows/listing.md) | 简体中文':'English | [简体中文](../../zh-CN/market/windows/listing.md)'}\n\n## ${h[1]}\n\nMSIX ${spec.baseline.package_version}, x64, \`${spec.baseline.revision}\`.\n\n## ${h[2]}\n\n\`\`\`text\n${l.product_name}\n\`\`\`\n\n## ${h[3]}\n\n\`\`\`text\n${l.description}\n\`\`\`\n\n## ${h[4]}\n\n${h[5]}\n\n## ${h[6]}\n\n${h[7]}\n\n`;
  l.features.forEach((f,i)=>s+=`${i+1}. ${f}\n`);
  s+=`\n## ${h[8]}\n\n| ${h[9]} | ${h[10]} |\n| --- | --- |\n| ${h[11]} | ${l.short_title} |\n| ${h[12]} | ${h[29]} |\n\n### ${h[13]}\n\n\`\`\`text\n${l.short_description}\n\`\`\`\n\n## ${h[14]}\n\n### ${h[15]}\n\n${h[16]}\n\n`;
  l.keywords.forEach(k=>s+=`- ${k}\n`);
  s+=`\n### ${h[17]}\n\n\`\`\`text\n${l.copyright}\n\`\`\`\n\n### ${h[18]}\n\n${h[19]}\n\n### ${h[20]}\n\n\`\`\`text\n${l.developer}\n\`\`\`\n\n## ${h[21]}\n\n| ${h[9]} | URL |\n| --- | --- |\n| ${h[22]} | ${spec.links.website} |\n| ${h[23]} | ${spec.links.support} |\n| ${h[24]} | ${l.privacy_url} |\n| ${h[25]} | ${spec.links.license} |\n\n## ${h[26]}\n\n${l.system_requirements.join('\n')}\n\n## ${h[27]}\n\n${h[28]}\n\n`;
  l.promotions.forEach(p=>s+=`### ${p.id}\n\n[1920 × 1080](images/promotional/${p.id}-1920x1080.png), [3840 × 2160](images/promotional/${p.id}-3840x2160.png)\n\n\`\`\`text\n${p.caption}\n\`\`\`\n\n`);
  if (captures) {
    s += `## ${chinese?'应用截图与融合宣传图':'App screenshots and combined artwork'}\n\n${chinese?'完整界面保持默认窗口大小。商店版按 1:1 放置窗口像素，仅添加中性留白，建议用于截图栏。融合宣传图加入品牌版式，供宣传用途。':'The complete app views retain the default window size. Store exports place window pixels at 1:1 with neutral padding; use them for screenshot fields. Combined artwork adds a brand layout for promotional use.'}\n\n`;
    for (const p of l.promotions) {
      const capture = captures.images.find(c => c.locale === key && c.scene === p.id);
      s += `### ${p.id}\n\n[${chinese?'默认窗口':'Default window'}](images/screenshots/${p.id}.png), [${chinese?'商店截图':'Store screenshot'}](images/store-screenshots/${p.id}.png), [1920 × 1080](images/combined/${p.id}-1920x1080.png), [3840 × 2160](images/combined/${p.id}-3840x2160.png)\n\n\`\`\`text\n${capture.caption}\n\`\`\`\n\n`;
    }
  }
  return s.trimEnd()+'\n';
}
async function main() {
  validateCopy(); const entries=[];
  for (const job of jobs) {
    const destination=path.join(ROOT,job.file); const source_sha256=hash(job.source);
    if (!check && !job.input) {
      fs.mkdirSync(path.dirname(destination),{recursive:true});
      if (job.canvas) {
        await sharp({create:{...job.canvas,channels:3}}).composite([{input:path.join(ROOT,job.capture.file),left:job.left,top:job.top}]).png({compressionLevel:9}).toFile(destination);
      } else await sharp(Buffer.from(job.source)).png({compressionLevel:9}).toFile(destination);
    }
    const bytes=fs.readFileSync(destination); const meta=await sharp(bytes).metadata();
    if (meta.format!=='png' || meta.width!==job.width || meta.height!==job.height || bytes.length>5000000) throw Error(`Invalid image ${job.file}`);
    if (!job.transparent && !(await sharp(bytes).stats()).isOpaque) throw Error(`Unexpected transparency ${job.file}`);
    if (job.input) {
      const capture = captures.images.find(c => c.file === job.file);
      if (hash(await sharp(bytes).removeAlpha().raw().toBuffer()) !== capture.pixels_sha256) throw Error(`Changed UI pixels ${job.file}`);
    }
    if (job.canvas) {
      if (meta.width < 1366 || meta.height < 768) throw Error(`Store screenshot canvas is too small ${job.file}`);
      const pixels = await sharp(bytes).extract({left:job.left,top:job.top,width:job.capture.width,height:job.capture.height}).removeAlpha().raw().toBuffer();
      if (hash(pixels) !== job.capture.pixels_sha256) throw Error(`Store screenshot scaled or changed UI pixels ${job.file}`);
    }
    if (job.role === 'screenshot-artwork' && job.width === 1920) {
      const placement = screenshotPlacement(job.capture);
      if (placement.scale !== 1) throw Error(`1080p artwork must preserve default window size ${job.file}`);
      const pixels = await sharp(bytes).extract({left:placement.imageX,top:placement.imageY,width:job.capture.width,height:job.capture.height}).removeAlpha().raw().toBuffer();
      if (hash(pixels) !== job.capture.pixels_sha256) throw Error(`1080p artwork changed default window pixels ${job.file}`);
    }
    const entry={file:job.file,width:job.width,height:job.height,role:job.role,locale:job.locale,source_sha256,sha256:hash(bytes)};
    if (check) {
      const old=previous.assets.find(a=>a.file===job.file);
      if (!old || JSON.stringify(old)!==JSON.stringify(entry)) throw Error(`Stale or changed asset ${job.file}`);
    }
    entries.push(entry);
  }
  if (check && previous.assets.length!==jobs.length) throw Error('Unexpected asset inventory entries');
  if (!check) fs.writeFileSync(indexFile,JSON.stringify({schema_version:1,brand_source:'design/assets/brand/xdzip.svg',assets:entries},null,2)+'\n');
  for(const [key,l] of Object.entries(spec.locales)) {
    const destination=path.join(ROOT,key==='en-US'?en:zh,'listing.md');
    const content=listingMarkdown(key,l);
    if(check) {if(fs.readFileSync(destination,'utf8')!==content) throw Error(`Stale listing copy ${key}`);}
    else fs.writeFileSync(destination,content);
  }
  console.log(`${check?'Validated':'Rendered'} ${jobs.length} PNG assets; bilingual field limits and source hashes passed.`);
}
main().catch(e=>{console.error(e.message);process.exitCode=1;});
