#!/usr/bin/env node
/* Reproduce a captured top border and desktop colors behind rounded corners. */
const assert = require('node:assert/strict');
const sharp = require('./windows-store-sharp.cjs');
const {makeCutout, promotionalLayer, composite, assertOverlay} = require('./windows-store-image-utils.cjs');

async function checkScreenshotShadow() {
  const width=90,height=65;
  const color='#f8dc73';
  const source=await sharp({create:{width,height,channels:3,background:color}}).png().toBuffer();
  const round=await makeCutout(source,width,height,96,{top_trim_pixels:0,corner_radius_logical:20});
  const box={imageWidth:width,imageHeight:height,imageX:100,imageY:100};
  const artwork={shadow_blur:6,shadow_offset_y:8};
  const canvas={width:1920,height:1080};
  const existing=await promotionalLayer(round.png,box,1,artwork,canvas);
  const fallback=await promotionalLayer(source,box,1,artwork,canvas);
  async function pixel(layer,x,y) {
    return [...await sharp(layer.png).extract({left:x-layer.left,top:y-layer.top,width:1,height:1})
      .ensureAlpha().raw().toBuffer()];
  }
  assert.ok((await pixel(existing,105,105))[3]<255, 'The native corner must retain its shape');
  assert.equal((await pixel(fallback,105,105))[3],255, 'Opaque captures must use the declared fallback radius');
  assert.deepEqual(await pixel(existing,145,130),[248,220,115,255], 'Shadow must remain behind the UI');
  const lowerShadow=(await pixel(existing,145,173))[3];
  assert.ok(lowerShadow>0 && lowerShadow<46, 'The window must cast a soft shadow beyond its lower edge');
  assert.equal((await pixel(existing,existing.left,existing.top))[3],0,'Padded shadow must reach clear alpha');
  const base=await sharp({create:{...canvas,channels:3,background:'#444455'}}).png().toBuffer();
  const finished=await composite(base,existing.png,existing.left,existing.top);
  await assertOverlay(finished,existing.png,existing.left,existing.top,base);
}

async function main() {
  const width=42,height=38,radius=6;
  const original=Buffer.alloc(width*height*4);
  for (let y=0;y<height;y++) for (let x=0;x<width;x++) {
    const p=(y*width+x)*4;
    const corner=(y<=radius || y>=height-radius) && (x<radius || x>=width-radius);
    const rgb=y===0?[255,0,255]:corner?[23,67,12]:[210+y,225,y+x];
    original.set([...rgb,255],p);
  }
  const png=await sharp(original,{raw:{width,height,channels:4}}).png().toBuffer();
  const result=await makeCutout(png,width,height,96,{top_trim_pixels:1,corner_radius_logical:radius});
  assert.deepEqual(result.crop,{left:0,top:1,width,height:height-1});
  const cleaned=await sharp(result.png).ensureAlpha().raw().toBuffer();
  for(let y=0;y<result.height;y++) for(let x=0;x<width;x++) {
    const p=(y*width+x)*4;
    const corner=(y<radius || y>=result.height-radius) && (x<radius || x>=width-radius);
    const sampleX=corner?(x<radius?radius:width-radius-1):x;
    const source=((y+1)*width+sampleX)*4;
    if(cleaned[p+3]===0) assert.deepEqual([...cleaned.subarray(p,p+3)],[0,0,0]);
    else assert.deepEqual(cleaned.subarray(p,p+3),original.subarray(source,source+3),`RGB changed outside corner cleanup at ${x},${y}`);
    if(!corner) assert.equal(cleaned[p+3],255);
  }
  for(const [x,y]of[[0,0],[width-1,0],[0,result.height-1],[width-1,result.height-1]]) {
    assert.equal(cleaned[(y*width+x)*4+3],0,'Desktop corner must be transparent');
  }
  for(const background of ['#ffffff','#101820']) {
    const composed=await sharp({create:{width,height:result.height,channels:3,background}})
      .composite([{input:result.png}]).raw().toBuffer();
    const expected=Buffer.from(background.slice(1),'hex');
    assert.deepEqual(composed.subarray(0,3),expected,'Old desktop color leaked into corner');
  }
  const framed=await makeCutout(png,width,height,96,{top_trim_pixels:1,corner_radius_logical:radius,border_pixels:1,border_color:'#C8D4E6'});
  assert.equal(framed.width,result.width);
  assert.equal(framed.height,result.height);
  const pixels=await sharp(framed.png).ensureAlpha().raw().toBuffer();
  for(let y=0;y<result.height;y++) for(let x=0;x<width;x++) {
    const p=(y*width+x)*4;
    assert.equal(pixels[p+3],cleaned[p+3],'Border changed rounded transparency');
    const corner=(y<radius || y>=result.height-radius) && (x<radius || x>=width-radius);
    if(x>0 && x<width-1 && y>0 && y<result.height-1 && !corner) {
      assert.deepEqual(pixels.subarray(p,p+4),cleaned.subarray(p,p+4),'Border changed UI away from perimeter');
    }
  }
  for(const [x,y]of[[width/2,0],[0,Math.floor(result.height/2)],[width-1,Math.floor(result.height/2)],[width/2,result.height-1]]) {
    const p=(y*width+x)*4;
    assert.deepEqual([...pixels.subarray(p,p+4)],[200,212,230,255],'Border missing or color differs');
  }
  await checkScreenshotShadow();
  console.log('PASS: clean window alpha, unchanged UI pixels and shadows derived from the screenshot shape.');
}
main().catch(error=>{console.error(error);process.exitCode=1});
