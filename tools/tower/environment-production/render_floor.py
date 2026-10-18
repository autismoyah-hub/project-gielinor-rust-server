"""Enhanced is the default. Matched Classic and measured Intel Lite comparisons."""
from pathlib import Path
import os,json,subprocess,time
from PIL import Image,ImageDraw,ImageFont
T=Path(__file__).resolve().parent;S=T.parents[2];E=S.parent;R=E/'cinderworks-environment-private'
VIEWS={'arrival':(56,56,0,1000,'matched'),'arrival-reverse':(56,56,8192,1000,'matched'),
 'workshop':(40,54,15000,1400,'player'),'slate':(40,60,1000,1200,'player'),'grove':(40,71,15000,1400,'player'),
 'waterworks':(70,58,15000,1400,'player'),'furnace-threshold':(56,65,0,1000,'matched'),
 'east-ash-court':(72,40,0,1400,'player'),'west-bone-court':(40,40,1500,1400,'player'),
 'imp-gallery':(72,72,0,1100,'player'),'warden-foundry':(56,72,0,1500,'player'),
 'reward':(55,53,8192,850,'player'),'departure':(62,60,12288,950,'player'),'processional-hall':(56,40,0,1000,'matched'),
 'whole-floor':(56,56,15300,11000,'floor-iso')}
if __name__=='__main__':
    out=R/'floor-review-v1';out.mkdir(exist_ok=True);reports=[]
    for mode,adapter,names in [('gpu','1',list(VIEWS)),('classic','1',['arrival','workshop','waterworks','imp-gallery','warden-foundry','whole-floor']),('lite','0',['arrival','workshop','warden-foundry'])]:
        for name in names:
            x,z,yaw,distance,preset=VIEWS[name];path=out/(name+'-'+mode+'.bmp');env=os.environ.copy();env.update(TOWER_ASSET_PACKAGE=str(R/'package-v1'),TOWER_RENDERER=mode,TOWER_GPU_ADAPTER=adapter,TOWER_REVIEW_FRAMES='20')
            before=time.monotonic();run=subprocess.run([str(E/'build/enhanced/release/cinderworks-review.exe'),str(E/'live-client-cache-v1'),str(R/'07-whole-floor-refinement/prepared.scene'),str(path),str(x),str(z),str(yaw),str(distance),preset,'runtime'],env=env,capture_output=True,text=True)
            (out/(name+'-'+mode+'.log')).write_text(run.stdout+run.stderr);run.check_returncode();metadata=json.loads(Path(str(path)+'.renderer.json').read_text());assert metadata['mode']==mode
            if mode!='classic':assert metadata['enhanced']and('Intel'if adapter=='0'else'NVIDIA')in metadata['stats']['adapter']
            Image.open(path).save(path.with_suffix('.png'));reports.append({'view':name,'mode':mode,'elapsed_seconds':time.monotonic()-before,'renderer':metadata,'camera':[x,z,yaw,distance,preset]});print('FLOOR_RENDER',name,mode,round(time.monotonic()-before,2),flush=True)
    (out/'render-report.json').write_text(json.dumps(reports,indent=2)+'\n')
    sheet=Image.new('RGB',(1800,5*430),'#111b20');draw=ImageDraw.Draw(sheet);font=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',22)
    for i,name in enumerate(VIEWS):
        im=Image.open(out/(name+'-gpu.png')).convert('RGB');im.thumbnail((590,390));x=(i%3)*600;y=(i//3)*430;sheet.paste(im,(x+(600-im.width)//2,y+35));draw.text((x+10,y+3),name,font=font,fill='#e1c38f')
    sheet.save(out/'enhanced-contact.png')
