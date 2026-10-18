"""Common-crop contact sheets; full fixed-coordinate frames remain authoritative."""
from pathlib import Path
from PIL import Image,ImageChops,ImageDraw
def contact(folders,labels,out,views=('side','quarter','front'),width=450,height=320):
    panel=Image.new('RGB',(width*len(folders),height*len(views)),'#171b20');d=ImageDraw.Draw(panel)
    for row,view in enumerate(views):
        ims=[Image.open(Path(p)/(view+'.png')).convert('RGB')for p in folders]
        bounds=[ImageChops.difference(im,Image.new('RGB',im.size,im.getpixel((0,0)))).getbbox()for im in ims]
        box=(min(b[0]for b in bounds)-5,min(b[1]for b in bounds)-5,max(b[2]for b in bounds)+5,max(b[3]for b in bounds)+5)
        for col,im in enumerate(ims):
            im=im.crop(box);im.thumbnail((width-10,height-40));panel.paste(im,(col*width+(width-im.width)//2,row*height+30));d.text((col*width+8,row*height+8),view+' / '+labels[col],fill='white')
    Path(out).parent.mkdir(parents=True,exist_ok=True);panel.save(out)
if __name__=='__main__':
    import sys
    contact(sys.argv[2:],['Authentic','Previous','Candidate'][:len(sys.argv)-2],sys.argv[1])
