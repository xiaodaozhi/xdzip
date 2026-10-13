#!/usr/bin/env python3
"""Publish unchanged evidence with equal-width viewing and an adjustable overlay."""
import argparse
import base64
import html
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
QA = ROOT / 'docs/qa/qt-fidelity'
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, default=ROOT / 'dist/xdzip-qt-fidelity.html')
args = parser.parse_args()


def url(path, mime='image/png'):
    return f'data:{mime};base64,' + base64.b64encode(path.read_bytes()).decode()


report = json.loads((QA / 'measurements.json').read_text(encoding='utf-8'))
sources = [
    dict(id='qt', title='修正后的 Qt', note='真实 ZIP · Linux Qt 6.8.2 offscreen · 1280 × 777 · 非 macOS 截图',
         url=url(QA / 'after-light.png'), width=1280, height=777, x=0, y=0),
    dict(id='html', title='已冻结的 HTML', note='05d4100 · 设计示例数据 · 新品牌 / 目录树 B',
         url=url(QA / 'html-baseline.png'), width=1280, height=940*1280/1548, x=0, y=0),
    dict(id='art', title='高保真原稿', note='原始图片 · 旧品牌 / 旧目录树 · 仅裁去窗口外背景并等比显示',
         url=url(ROOT / 'design/hifi/screens/main-light.png'), width=1586*1280/1548,
         height=940*1280/1548, x=-20*1280/1548, y=-30*1280/1548),
    dict(id='before', title='修正前 Qt', note='05d4100 · macOS ARM64 offscreen · normal.zip · 1280 × 820',
         url=url(QA / 'before-macos.png'), width=1280, height=820, x=0, y=0),
    dict(id='dark', title='修正后的 Qt 深色', note='同一真实归档与选择 · 深色完整原稿尚未冻结',
         url=url(QA / 'after-dark.png'), width=1280, height=777, x=0, y=0),
]
checks = ''.join('<tr>' + ''.join(f'<td>{html.escape(str(c[k]))}</td>' for k in ['name', 'expected', 'actual']) + '</tr>'
                 for c in report['checks'] if c['name'].startswith('main-light/') and ('.height' in c['name'] or '.font' in c['name'] or 'Headers' in c['name']))
template = r'''<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>xdzip · Qt 界面校准验收</title><style>
@font-face{font-family:Review;src:url(__FONT__)}*{box-sizing:border-box}body{margin:0;background:#f3f5f9;color:#141827;font:15px/1.65 Review,system-ui,sans-serif}main{max-width:1360px;margin:auto;padding:20px}h1{font-size:25px;margin:0 0 10px}p{margin:8px 0 16px}.note{background:#fff;border:1px solid #dfe4ee;border-radius:12px;padding:16px}.controls{display:flex;flex-wrap:wrap;gap:12px;margin:20px 0;align-items:center}label{display:flex;align-items:center;gap:7px}select,input,button{font:inherit}select,button{padding:7px;border:1px solid #d0d7e4;border-radius:6px;background:white;color:inherit}article{min-width:0;margin-bottom:18px;background:white;border:1px solid #dfe4ee;border-radius:10px;overflow:hidden}.caption{padding:10px 14px}.caption small{display:block;color:#626b86}.scroll{overflow:auto}.frame{position:relative;overflow:hidden;background:repeating-linear-gradient(135deg,#e8edf5 0 10px,#f7f9fc 10px 20px);flex-shrink:0}.clip{position:absolute;overflow:hidden;width:1280px}.shot{position:absolute;max-width:none}.canvas{position:absolute;width:1280px;height:820px;transform-origin:top left}.table{overflow:auto}table{border-collapse:collapse;width:100%;background:white;font-size:13px}td,th{padding:8px;text-align:left;border:1px solid #dfe4ee;white-space:nowrap}summary{cursor:pointer}pre{white-space:pre-wrap;font-size:11px}.legend{color:#626b86}#export{display:none}.capture main{display:none}.capture #export{display:block;padding:22px;width:1324px;background:#f3f5f9}.capture article{border-radius:0}.capture .caption{font-size:19px}
</style><main><h1>Qt 界面校准验收</h1>
<p>以同一 1280 逻辑像素宽度对照，保持图片比例；原稿、HTML 和 Qt 截图保留原始字节。</p>
<div class="note">独立几何与字体检查 <b>__COUNT__ 项通过</b>，共享控件检查 72 项通过。这个数量是约束检查数，不是视觉还原率。<br>原稿窗口 1548 × 940 → 1280 × 777.26；Qt 窗口取整数 1280 × 777。字号按 Qt 整数像素取整。<br>Qt 正在浏览真实 ZIP；图片解码、修改时间、多归档会话、安全解压与新建尚未实现。因此没有示例照片、伪造日期、装饰性第二标签，也没有把禁用按钮画成可执行状态。macOS 系统窗口装饰需要原生桌面证据。</div>
<div class="controls"><label>查看<select id="mode"><option value="cards">逐张对照</option><option value="overlay">叠加检查</option></select></label><label>区域<select id="region"><option value="all">完整窗口</option><option value="top">标题栏 / 工具栏</option><option value="list">列表</option><option value="sidebar">侧栏</option><option value="inspector">预览面板</option></select></label><label>倍率<select id="zoom"><option value="fit">适应屏幕</option><option value="1">100% 逻辑尺寸</option><option value="2">200%</option></select></label><label>叠加对象<select id="pair"><option value="html,qt">HTML + Qt</option><option value="art,qt">原稿 + Qt</option><option value="before,qt">修正前 + 修正后</option></select></label><label>上层不透明度<input id="opacity" type="range" min="0" max="100" value="50"></label></div>
<p class="legend" id="region-note"></p><div id="viewer"></div>
<details><summary>独立测量明细（部分；完整记录见 measurements.json）</summary><div class="table"><table><thead><tr><th>检查</th><th>冻结基准</th><th>Qt 实测</th></tr></thead><tbody>__CHECKS__</tbody></table></div></details>
<p>保持的设计修订：新品牌与目录树 B。运行字体与 HTML 字体同源；未收录的归档名称字形回退至系统字体。深色强调色为保证白字对比度采用 #3264E9。</p>
<details><summary>字体许可：SIL Open Font License 1.1</summary><pre>__LICENSE__</pre></details></main><div id="export"></div>
<script>
const sources=__SOURCES__,regions={all:[0,0,1280,820],top:[0,0,1280,174],list:[240,112,718,620],sidebar:[0,112,241,620],inspector:[958,112,322,620]};
function frame(items,box,scale){const f=document.createElement('div');f.className='frame';f.style.width=box[2]*scale+'px';f.style.height=box[3]*scale+'px';const c=document.createElement('div');c.className='canvas';Object.assign(c.style,{left:-box[0]*scale+'px',top:-box[1]*scale+'px',transform:`scale(${scale})`});items.forEach((s,i)=>{const clip=document.createElement('div');clip.className='clip';clip.style.height=s.height+'px';if(i)clip.style.opacity=document.querySelector('#opacity').value/100;const img=document.createElement('img');img.className='shot';img.src=s.url;img.alt=s.title;Object.assign(img.style,{left:s.x+'px',top:s.y+'px',width:s.width+'px'});clip.append(img);c.append(clip)});f.append(c);return f}
function card(s){const a=document.createElement('article'),label=document.createElement('div');label.className='caption';const b=document.createElement('b'),small=document.createElement('small');b.textContent=s.title;small.textContent=s.note;label.append(b,small);a.append(label);return a}
function render(){const region=document.querySelector('#region').value,box=regions[region],viewer=document.querySelector('#viewer'),zoom=document.querySelector('#zoom').value;viewer.replaceChildren();const overlay=document.querySelector('#mode').value==='overlay';document.querySelector('#region-note').textContent='共同坐标区域：'+box.join(', ')+'；条纹部分表示该截图没有内容。';const groups=overlay?[document.querySelector('#pair').value.split(',').map(id=>sources.find(s=>s.id===id))]:sources.map(s=>[s]);for(const items of groups){const a=card(items[0]);if(overlay)a.querySelector('b').textContent=items.map(s=>s.title).join(' + ');viewer.append(a);const scroll=document.createElement('div');scroll.className='scroll';a.append(scroll);scroll.append(frame(items,box,zoom==='fit'?Math.min(1,scroll.clientWidth/box[2]):+zoom))}}
window.exportRegion=(region)=>{document.body.classList.add('capture');const out=document.querySelector('#export');out.replaceChildren();const title=document.createElement('h1');title.textContent='xdzip · Qt / HTML / 原稿 · '+region;out.append(title);for(const s of sources.slice(0,3)){const a=card(s);a.append(frame([s],regions[region],1280/regions[region][2]));out.append(a)}};
document.querySelectorAll('select,input').forEach(n=>n.addEventListener('input',render));window.addEventListener('resize',render);render();
</script></html>'''
content = template.replace('__FONT__', url(ROOT / 'design/assets/fonts/xdzip-design-sans-sc.woff2', 'font/woff2'))
content = content.replace('__COUNT__', str(report['passed'])).replace('__CHECKS__', checks)
content = content.replace('__SOURCES__', json.dumps(sources, ensure_ascii=False))
content = content.replace('__LICENSE__', html.escape((ROOT / 'design/assets/fonts/OFL.txt').read_text(encoding='utf-8')))
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(content, encoding='utf-8')
print(args.output)
