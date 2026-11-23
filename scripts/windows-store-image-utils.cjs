const sharp = require('./windows-store-sharp.cjs');
const crypto = require('node:crypto');
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

async function makeCutout(bytes, width, height, dpi, processing) {
  const trim = processing.top_trim_pixels;
  const crop = {left:0,top:trim,width,height:height-trim};
  if (!Number.isInteger(trim) || trim < 0 || crop.width <= 0 || crop.height <= 0) throw Error('Invalid screenshot perimeter trim');
  const radius = Math.round(processing.corner_radius_logical * dpi / 96);
  if (radius < 0 || radius * 2 > Math.min(crop.width,crop.height)) throw Error('Invalid screenshot corner radius');
  const raw = await sharp(bytes).extract(crop).ensureAlpha().raw().toBuffer();
  const mask = Buffer.from(`<svg width="${crop.width}" height="${crop.height}"><rect width="100%" height="100%" rx="${radius}" fill="white"/></svg>`);
  const alpha = await sharp(mask).ensureAlpha().raw().toBuffer();
  // Native rounded edges are already blended with the desktop. Use the same
  // row's interior background in the corner squares before applying new alpha,
  // otherwise old desktop colors survive as a dark or colored fringe.
  for (let y=0; y<crop.height; y++) {
    if (y>=radius && y<crop.height-radius) continue;
    for (let x=0; x<crop.width; x++) {
      if (x>=radius && x<crop.width-radius) continue;
      const from=(y*crop.width+(x<radius?radius:crop.width-radius-1))*4;
      const to=(y*crop.width+x)*4;
      raw.copy(raw,to,from,from+3);
    }
  }
  for (let p=0; p<raw.length; p+=4) {
    raw[p+3] = alpha[p+3];
    if (!raw[p+3]) raw.fill(0,p,p+3);
  }
  const border = processing.border_pixels ?? 0;
  if (!Number.isInteger(border) || border < 0 || border*2 >= Math.min(crop.width,crop.height)) throw Error('Invalid screenshot border width');
  if (border) {
    if (!/^#[0-9a-f]{6}$/i.test(processing.border_color)) throw Error('Invalid screenshot border color');
    const color = Buffer.from(processing.border_color.slice(1),'hex');
    const inside = Buffer.from(`<svg width="${crop.width}" height="${crop.height}"><rect x="${border}" y="${border}" width="${crop.width-border*2}" height="${crop.height-border*2}" rx="${Math.max(0,radius-border)}" fill="white"/></svg>`);
    const innerAlpha = await sharp(inside).ensureAlpha().raw().toBuffer();
    // Color the inner perimeter while retaining the existing outer alpha.
    // Compositing a second rounded stroke would make edge pixels more opaque.
    for (let p=0; p<raw.length; p+=4) {
      if (!raw[p+3]) continue;
      const retained = Math.min(1,innerAlpha[p+3]/raw[p+3]);
      for (let c=0;c<3;c++) raw[p+c] = Math.round(raw[p+c]*retained+color[c]*(1-retained));
    }
  }
  const png = await sharp(raw,{raw:{width:crop.width,height:crop.height,channels:4}}).png({compressionLevel:9}).toBuffer();
  if (!(await sharp(png).ensureAlpha().raw().toBuffer()).equals(raw)) throw Error('Cutout PNG changed RGBA pixels');
  return {png,width:crop.width,height:crop.height,crop,radius,rgba_sha256:hash(raw)};
}

function placement(spec, capture, width, height) {
  const ratio = width / spec.visual.canvas[0];
  if (height !== spec.visual.canvas[1] * ratio) throw Error('Promotional canvas aspect differs');
  const [x,y,w,h] = spec.screenshot_artwork.frame.map(n=>Math.round(n*ratio));
  const scale = Math.min(1,w/capture.cutout_width,h/capture.cutout_height);
  const imageWidth = Math.min(capture.cutout_width,Math.floor(capture.cutout_width*scale));
  const imageHeight = Math.min(capture.cutout_height,Math.floor(capture.cutout_height*scale));
  return {imageWidth,imageHeight,imageX:x+Math.floor((w-imageWidth)/2),imageY:y+Math.floor((h-imageHeight)/2),scale};
}

async function screenshotLayer(bytes, box) {
  const meta = await sharp(bytes).metadata();
  if (box.imageWidth > meta.width || box.imageHeight > meta.height) throw Error('Screenshot upscaling is forbidden');
  if (box.imageWidth === meta.width && box.imageHeight === meta.height) return bytes;
  return sharp(bytes).resize(box.imageWidth,box.imageHeight,{fit:'fill',kernel:'lanczos3'}).png().toBuffer();
}

async function promotionalLayer(bytes, box, radius, artwork, canvas) {
  const info = await sharp(bytes).metadata();
  if (box.imageWidth > info.width || box.imageHeight > info.height) throw Error('Screenshot upscaling is forbidden');
  const {width, height} = info;
  const alpha = await sharp(bytes).ensureAlpha().extractChannel('alpha').raw().toBuffer();
  const clearlyRounded = [0, width - 1, width * (height - 1), width * height - 1].some(i => alpha[i] < 255);
  // Recorded window silhouettes and their anti-aliased borders stay intact. Only opaque captures need a cutout.
  const input = clearlyRounded ? bytes : (await makeCutout(bytes, width, height, 144, {
    top_trim_pixels: 0,
    corner_radius_logical: radius,
  })).png;
  const image = await screenshotLayer(input, box);

  const {imageWidth, imageHeight, imageX, imageY} = box;
  const ratio = canvas.width / 1920;
  const sigma = artwork.shadow_blur * ratio;
  const offset = Math.round(artwork.shadow_offset_y * ratio);
  if (!Number.isFinite(sigma) || sigma < .3 || !Number.isInteger(offset))
    throw Error('Invalid promotional shadow geometry');
  const padding = Math.ceil(4 * sigma);
  const imageTop = padding + Math.max(0, -offset);
  const shadowWidth = imageWidth + 2 * padding;
  const shadowHeight = imageHeight + 2 * padding + Math.abs(offset);
  const left = imageX - padding, top = imageY - imageTop;
  if (left < 0 || top < 0 || left + shadowWidth > canvas.width || top + shadowHeight > canvas.height)
    throw Error('Promotional screenshot shadow exceeds the canvas');

  // Blur the actual final-size window alpha, including its corner shape. The extra space keeps the shadow uncut.
  const windowAlpha = await sharp(image).ensureAlpha().extractChannel('alpha').raw().toBuffer();
  const mask = Buffer.alloc(shadowWidth * shadowHeight);
  for (let y = 0; y < imageHeight; y++) {
    windowAlpha.copy(mask, (y + imageTop + offset) * shadowWidth + padding, y * imageWidth, (y + 1) * imageWidth);
  }
  const raw = {width: shadowWidth, height: shadowHeight, channels: 1};
  const blurred = await sharp(mask, {raw}).blur(sigma).toColourspace('b-w').raw().toBuffer();
  const pixels = Buffer.alloc(blurred.length * 4);
  for (let i = 0; i < blurred.length; i++) {
    const opacity = Math.round(blurred[i] * .18);
    if (!opacity) continue;
    const p = i * 4;
    pixels[p] = 23;
    pixels[p+1] = 55;
    pixels[p+2] = 99;
    pixels[p+3] = opacity;
  }
  const png = await sharp(pixels, {raw: {...raw, channels: 4}})
    .composite([{input: image, left: padding, top: imageTop}]).png().toBuffer();
  return {png, left, top};
}

async function composite(background, layer, left, top) {
  return sharp(background).composite([{input:layer,left,top}]).png({compressionLevel:9}).toBuffer();
}

async function assertOverlay(output,layer,left,top,background) {
  const meta = await sharp(layer).metadata();
  const ui = await sharp(layer).ensureAlpha().raw().toBuffer();
  const actual = await sharp(output).extract({left,top,width:meta.width,height:meta.height}).removeAlpha().raw().toBuffer();
  const behind = await sharp(background).extract({left,top,width:meta.width,height:meta.height}).removeAlpha().raw().toBuffer();
  for (let p=0,q=0; p<ui.length; p+=4,q+=3) {
    for (let c=0;c<3;c++) {
      if (ui[p+3]===255 && actual[q+c]!==ui[p+c]) throw Error('Compositor altered opaque UI pixels');
      if (ui[p+3]===0 && actual[q+c]!==behind[q+c]) throw Error('Transparent corner contains border or background residue');
    }
  }
}

module.exports = {hash,makeCutout,placement,promotionalLayer,composite,assertOverlay};
