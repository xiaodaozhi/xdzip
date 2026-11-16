#!/usr/bin/env node
/* Reproduce a captured top border and desktop colors behind rounded corners. */
const assert = require('node:assert/strict');
const sharp = require('sharp');
const {makeCutout} = require('./windows-store-image-utils.cjs');

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
  console.log('PASS: top row removed, all four corners cleared on light/dark backgrounds, remaining UI RGB unchanged.');
}
main().catch(error=>{console.error(error);process.exitCode=1});
