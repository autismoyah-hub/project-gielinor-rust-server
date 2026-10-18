"""Build a private, portable relative-link gallery from an explicit manifest.

No image generation, cache extraction or site publication is performed here.
The manifest's evidence URLs must be relative to the output index.html.
"""
import argparse,json,html
from pathlib import Path

def build(manifest,out):
    data=json.loads(manifest.read_text(encoding='utf-8'));out.mkdir(parents=True,exist_ok=True)
    esc=html.escape
    parts=['<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>ASCENT · Production fidelity review</title><style>:root{color-scheme:dark;font:16px/1.6 system-ui;background:#10161b;color:#e5e4db}body{max-width:1440px;margin:auto;padding:28px}h1{font-size:40px;line-height:1.2}h2{margin-top:64px}p{max-width:1080px}a{color:#e7c18c}nav{display:flex;gap:24px;flex-wrap:wrap}figure{margin:0}img{max-width:100%;height:auto;background:#10161b}figcaption{color:#b9c2c4;font-size:14px}article{border:1px solid #39444b;padding:20px;margin:24px 0;border-radius:8px}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(290px,1fr));gap:20px}table{width:100%;border-collapse:collapse}td,th{text-align:left;padding:10px;border-bottom:1px solid #39444b}.notice{border-left:3px solid #dca863;padding:12px 20px;background:#20282e}details{margin:20px 0}summary{cursor:pointer;color:#e7c18c}code{overflow-wrap:anywhere}@media(max-width:700px){body{padding:16px}h1{font-size:30px}}</style><p>ASCENT · Native offline evidence · revision 239.1</p><h1>Original construction.<br>Native motion. Honest limits.</h1>']
    parts.append('<p>'+esc(data['intro'])+'</p><p class="notice">'+esc(data['notice'])+'</p><nav>')
    for s in data['sections']:parts.append('<a href="#'+esc(s['id'])+'">'+esc(s['title'])+'</a>')
    parts.append('</nav>')
    for s in data['sections']:
        parts.append('<section id="'+esc(s['id'])+'"><h2>'+esc(s['title'])+'</h2>')
        for note in s.get('notes',[]):parts.append('<p>'+esc(note)+'</p>')
        if s.get('table'):
            parts.append('<table><thead><tr>'+''.join('<th>'+esc(x)+'</th>'for x in s['table']['columns'])+'</tr></thead><tbody>')
            for row in s['table']['rows']:parts.append('<tr>'+''.join('<td>'+esc(str(x))+'</td>'for x in row)+'</tr>')
            parts.append('</tbody></table>')
        for c in s.get('cards',[]):
            parts.append('<article><h3>'+esc(c['title'])+'</h3><p>'+esc(c.get('note',''))+'</p><div class="grid">')
            for m in c.get('media',[]):
                url=esc(m['url'],quote=True);label=esc(m['label']);parts.append('<figure><a href="'+url+'" target="_blank"><img loading="lazy" src="'+url+'" alt="'+label+'"></a><figcaption>'+label+'</figcaption></figure>')
            parts.append('</div>')
            for link in c.get('links',[]):parts.append('<p><a href="'+esc(link['url'],quote=True)+'">'+esc(link['label'])+'</a></p>')
            parts.append('</article>')
        parts.append('</section>')
    parts.append('</html>');(out/'index.html').write_text(''.join(parts),encoding='utf-8')
    (out/'review-manifest.json').write_text(json.dumps(data,indent=2),encoding='utf-8')
    print('PRIVATE_PRODUCTION_REVIEW_READY',out/'index.html')

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('manifest',type=Path);p.add_argument('out',type=Path);a=p.parse_args();build(a.manifest,a.out)
