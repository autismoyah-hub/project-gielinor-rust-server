"""Rendered-image-only anvil comparison. No reference geometry is loaded."""
from run import *
from PIL import ImageChops
out=ROOT/'reference-comparison';out.mkdir(exist_ok=True);report={}
def mask(im):
    rgb=im.convert('RGB');bg=Image.new('RGB',rgb.size,rgb.getpixel((0,0)));return ImageChops.difference(rgb,bg).convert('L').point(lambda x:255 if x else 0)
for version in ['first','refined','final','final2']:
    report[version]={}
    for view in ['front','side','rear','quarter','gameplay']:
        ref=Image.open(REFERENCE/(view+'.bmp')).convert('RGB');own=Image.open(ROOT/'runs'/('anvil-'+version)/(view+'.png')).convert('RGB');a,b=mask(ref),mask(own);intersection=ImageChops.multiply(a,b);union=ImageChops.lighter(a,b);iou=sum(intersection.getdata())/sum(union.getdata());report[version][view]=iou
        if version=='final2':
            ref.save(out/('reference-'+view+'.png'));own.save(out/('authored-'+view+'.png'));Image.blend(ref,own,.5).save(out/('overlay-'+view+'.png'))
            bounds=union.getbbox();x0,y0,x1,y1=bounds;box=(max(0,x0-25),max(0,y0-25),min(ref.width,x1+25),min(ref.height,y1+25));panel=Image.new('RGB',((box[2]-box[0])*2,box[3]-box[1]));panel.paste(ref.crop(box),(0,0));panel.paste(own.crop(box),(box[2]-box[0],0));panel.save(out/('pair-'+view+'.png'))
(out/'silhouette.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2))
