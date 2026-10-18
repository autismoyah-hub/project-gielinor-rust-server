"""Review complete owned native action clips and an explicitly matched old study.

Images come from native playback. This tool only labels/resizes those images;
it does not render, interpolate, retime or fit animation.
"""
import argparse,json
from pathlib import Path
from PIL import Image,ImageDraw

def review(root,out,previous=None):
    if out.exists()and any(out.iterdir()):raise ValueError('Preserve prior reviews; choose an empty output')
    out.mkdir(parents=True,exist_ok=True);rows=[]
    for folder in sorted((root/'motion').iterdir()):
        if not folder.is_dir():continue
        ticks=sorted(int(p.stem.split('-')[1])for p in folder.glob('pose-*.bmp'))
        if not ticks:continue
        frames=[]
        for tick in ticks:
            im=Image.open(folder/f'pose-{tick:04}.bmp').convert('RGB');im.thumbnail((560,520));frame=Image.new('RGB',(560,550),(16,21,26));frame.paste(im,(0,28));ImageDraw.Draw(frame).text((12,8),f'{folder.name} | native tick {tick}',fill='white');frames.append(frame)
        durations=[max(20,(ticks[i+1]-t)*20)if i+1<len(ticks)else 800 for i,t in enumerate(ticks)]
        frames[0].save(out/(folder.name+'.gif'),save_all=True,append_images=frames[1:],duration=durations,loop=0)
        common=[]
        if previous and(previous/'motion'/folder.name).is_dir():
            common=[t for t in ticks if(previous/'motion'/folder.name/f'pose-{t:04}.bmp').exists()];selected=sorted({common[round(i*(len(common)-1)/6)]for i in range(7)})
            sheet=Image.new('RGB',(280*len(selected),584),(16,21,26));draw=ImageDraw.Draw(sheet)
            for row,(label,base)in enumerate([('Previous',previous),('Revised',root)]):
                for col,t in enumerate(selected):
                    im=Image.open(base/'motion'/folder.name/f'pose-{t:04}.bmp');im.thumbnail((280,260));sheet.paste(im,(col*280,row*292+28));draw.text((col*280+6,row*292+6),f'{label} | tick {t}',fill='white')
            sheet.save(out/(folder.name+'-old-new.png'))
        rows.append(dict(action=folder.name,native_ticks=ticks,matched_previous_ticks=common,retimed=False))
    (out/'review.json').write_text(json.dumps(dict(actions=rows,source='Native BMPs only; per-asset framing retained, no image registration',artistic_acceptance=False),indent=2),encoding='utf-8')

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('root',type=Path);p.add_argument('out',type=Path);p.add_argument('--previous',type=Path);a=p.parse_args();review(a.root,a.out,a.previous)
