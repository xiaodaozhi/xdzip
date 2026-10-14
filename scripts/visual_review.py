"""Offline viewer builder; all evidence is read from the current ignored output."""
import base64
import html
import json


def build_review(source, output, title):
    report = json.loads((source/'measurements.json').read_text())
    figures = []
    for name, label in [('reference', '高保真原稿'), ('after', '本次实际渲染'), ('difference', '像素差异')]:
        data = base64.b64encode((source/(name+'.png')).read_bytes()).decode()
        figures.append(f'<figure><figcaption>{label}</figcaption><img src="data:image/png;base64,{data}" alt="{label}"></figure>')
    body = '<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">'
    body += '<style>body{margin:24px;background:#f4f6fa;color:#182033;font:14px/1.6 system-ui}img{max-width:100%;height:auto}figure{margin:24px 0}pre{white-space:pre-wrap;overflow-wrap:anywhere}</style>'
    body += '<h1>'+html.escape(title)+'</h1><p>真实生成工件；本文件只在本地或 CI 中保留。</p>'+''.join(figures)
    body += '<details><summary>当前量测记录</summary><pre>'+html.escape(json.dumps(report,ensure_ascii=False,indent=2))+'</pre></details></html>'
    output.parent.mkdir(parents=True,exist_ok=True)
    output.write_text(body)
    print(output)
