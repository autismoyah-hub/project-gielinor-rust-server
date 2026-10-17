"""Compile isolated authored design JSON into native-library scene input.

Only generated diagrams (original geometric drawing) may be published by default.
Cache-rendered output stays local pending separately documented media rights.
"""
import argparse
import collections
import hashlib
import json
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

MATERIALS = {'stone': (377,545), 'dark': (364,1245), 'forge': (362,1211),
             'moss': (212,571), 'earth': (321,649), 'ash': (242,620), 'water': (187,566)}
SIDES = [(-1,0,0),(0,1,1),(1,0,2),(0,-1,3)]
def rect(r):
    x,z,xx,zz=r
    return {(a,b)for a in range(x,xx)for b in range(z,zz)}
def compile_scene(c):
    chunks={(x,int(z)):'dark'for z,xs in c['rows'].items()for x in xs}
    for p in c['water_chunks']:chunks[tuple(p)]='water'
    for name,kind in [('hub','stone'),('workshop','forge'),('grove','moss'),('provisions','earth')]:
        x,z=c[name]
        for zx in range(x//8-1,x//8+2):
            for zz in range(z//8-1,z//8+2):
                if (zx,zz)in chunks and chunks[zx,zz]!='water' and abs(zx*8+4-x)+abs(zz*8+4-z)<16:chunks[zx,zz]=kind
    for boss,kind in [('imp','ash'),('warden','forge')]:
        lo,hi=c[boss]['min'],c[boss]['max']
        for x in range(lo[0]//8,hi[0]//8+1):
            for z in range(lo[1]//8,hi[1]//8+1):chunks[x,z]=kind
    loaded=set().union(*(rect([x*8,z*8,x*8+8,z*8+8])for x,z in chunks))
    water=set().union(*(rect([x*8,z*8,x*8+8,z*8+8])for (x,z),kind in chunks.items()if kind=='water'))
    dry=loaded-water
    # A real wall on the inner flat tile guards the one-tile height/void rim.
    walk={p for p in dry if all((p[0]+dx,p[1]+dz)in dry for dx,dz,_ in SIDES)}
    walls=collections.defaultdict(set)
    wall_kind={}
    for x,z in walk:
        for dx,dz,a in SIDES:
            if (x+dx,z+dz)not in walk:
                walls[x,z].add(a);wall_kind[x,z]=6143 if a==3 or c['id']=='b'else 6145
    for enclosure in c['enclosures']:
        cells=set().union(*(rect(r)for r in enclosure['rects']))
        gates=set().union(*(rect(r)for r in enclosure['gates']))
        for x,z in cells&walk:
            for dx,dz,a in SIDES:
                if (x+dx,z+dz)in cells or (x,z)in gates:continue
                if enclosure['ruined'] and (x+z)%7 in [0,1,2]:continue
                walls[x,z].add(a);wall_kind[x,z]=6143 if a==3 or enclosure['ruined']else 6145
    locs=[]
    for (x,z),sides in sorted(walls.items()):
        if len(sides)==1:locs.append([x,z,0,wall_kind[x,z],0,next(iter(sides))])
        elif len(sides)==2 and any({a,(a+1)%4}==sides for a in range(4)):
            a=next(a for a in range(4)if {a,(a+1)%4}==sides);locs.append([x,z,0,wall_kind[x,z],2,a])
        else:
            # Multiple walls at an internal junction use their opposite cells,
            # leaving one publishable wall layer per tile, not overlapping walls.
            for a in sides:
                dx,dz,_=SIDES[a];locs.append([x+dx,z+dz,0,wall_kind[x,z],0,(a+2)%4])
    assets=[]
    def obj(label,id,x,z,shape=10,angle=0):
        locs.append([x,z,0,id,shape,angle]);assets.append({'label':label,'id':id,'x':x,'z':z,'shape':shape,'rotation':angle})
    e=c['extra'];hx,hz=c['hub'];wx,wz=c['workshop'];gx,gz=c['grove']
    for label,id in [('bank',18491),('altar',409),('chest',57694),('range',9736)]:obj(label,id,*e[label])
    obj('entry/exit portal',57692,*e['portal'],shape=0)
    obj('plaque',909,e['chest'][0]-1,e['chest'][1],shape=4,angle=1)
    obj('sanctuary hearth',16811,hx,hz)
    obj('altar fire',16811,e['altar'][0]-2,e['altar'][1]+2)
    obj('workshop forge landmark',18526,wx+3,wz-3)
    for dx in [-4,4]:
        if (hx+dx,hz-3)in walk:obj('sanctuary dwarf monument',568,hx+dx,hz-3)
    for dz in [-1,3]:obj('wheel pier',7017,e['wheel'][0]-2,e['wheel'][1]+dz)
    obj('anvil',2097,wx,wz-3);obj('smelting furnace',37947,wx-3,wz-3)
    obj('tin',11360,wx-3,wz+3);obj('copper',10943,wx+4,wz+3)
    obj('wheel',18511,*e['wheel'])
    for dx,dz in [(-2,-1),(2,2),(-3,3)]:
        if (gx+dx,gz+dz)in walk:obj('tree reserve',1276,gx+dx,gz+dz)
    for boss in ['imp','warden']:
        b=c[boss];sx,sz=b['spawn'];lo,hi=b['min'],b['max']
        if boss=='warden':
            for x in [lo[0]+1,lo[0]+5,lo[0]+10]:
                obj('foundry forge',18526,x,lo[1]-2 if b.get('furnace_side')=='south' else hi[1]+1,angle=3 if b.get('furnace_side')=='south' else 1)
            ax,az=b['approach'];dz=1 if b.get('furnace_side')=='south' else -1
            for dx in [-3,3]:obj('foundry threshold pier',7016,ax+dx,az+dz)
            for dx,dz in [(-3,-1),(-3,2),(4,-1),(4,2)]:obj('coolant',30124,sx+dx,sz+dz)
        else:
            obj('Imp boiler',18526,sx-1,hi[1]+1,angle=1)
            obj('boiler inlet',9116,sx-3,hi[1]+1);obj('boiler outlet',9120,sx+2,hi[1]+1,angle=2)
    for x,z in e['ruins']:obj('collapse outside court',6915,x,z)
    # Sparse piers and devotional landmarks frame thresholds, not combat cells.
    for x,z in [(hx-3,hz+4),(hx+3,hz+4)]:
        if (x,z)in walk:obj('hub pier',7016,x,z)
    points={'entry':e['entry'],'recovery':e['recovery'],'hub':[hx-1,hz],
            'grove approach':[gx,gz-2],'provisions':c['provisions'],
            'Imp approach':c['imp']['approach'],'Warden approach':c['warden']['approach']}
    for k,v in c['views'].items():points['view-'+k]=v[:2]
    for i,p in enumerate(c['route']):points['route-'+str(i)]=p
    lines=['# OFFLINE PROPOSAL ONLY; plane0; native cache chunks; no gameplay installation']
    for (x,z),kind in sorted(chunks.items()):
        sx,sz=MATERIALS[kind];rot=0 if kind in ['water','dark','ash','earth','moss']else(x+z)%4
        lines.append(f'T {x} {z} 0 {sx} {sz} 0 {rot}')
    lines += ['L '+' '.join(map(str,row))for row in locs]
    lines += [f'P {name.replace(" ","_")} {x} {z}'for name,(x,z)in points.items()]
    for boss in ['imp','warden']:
        b=c[boss];lines.append('A '+boss+' '+' '.join(map(str,b['min']+b['max']+b['spawn']+[1 if boss=='imp'else 2])))
    for i,(x,z)in enumerate(e['fishing']):lines.append(f'F fish{i} {x} {z}')
    info={'id':c['id'],'title':c['title'],'chunks':len(chunks),'loaded_tiles':len(loaded),'water_tiles':len(water),'planned_inner_walkable_tiles':len(walk),'points':points,'assets':assets,'materials':{f'{x},{z}':kind for(x,z),kind in chunks.items()},'scope':'Detached proposal, actual cache material chunks and wall/location definitions. No actors, live scripts, personal state or saved accounts. Static feasibility checked separately.'}
    return '\n'.join(lines)+'\n',info,walk,walls

def schematic(c,info,walk,walls,path):
    # Entirely original geometry and typography. No cache pixels or models.
    im=Image.new('RGB',(1200,1300),'#141c24');d=ImageDraw.Draw(im)
    font=ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf',19);title=ImageFont.truetype('C:/Windows/Fonts/segoeuib.ttf',31)
    d.text((42,25),c['title'],font=title,fill='#f4d08a');d.text((42,73),'ORIGINAL LAYOUT DIAGRAM · concept only · north up · same tile scale',font=font,fill='#b8c7cf')
    scale=21;ox=80;oy=120
    def box(x,z):return (ox+(x-32)*scale,oy+(79-z)*scale,ox+(x-31)*scale,oy+(80-z)*scale)
    colors={'stone':'#576673','dark':'#384853','forge':'#72594c','moss':'#47635b','earth':'#77694c','ash':'#615b68','water':'#2f718a'}
    for xy,kind in info['materials'].items():
        zx,zz=map(int,xy.split(','))
        for x in range(zx*8,zx*8+8):
            for z in range(zz*8,zz*8+8):d.rectangle(box(x,z),fill=colors[kind])
    for (x,z),sides in walls.items():
        l,t,r,b=box(x,z)
        for a in sides:d.line([(l,b,l,t),(l,t,r,t),(r,t,r,b),(l,b,r,b)][a],fill='#d2c3a5',width=3)
    def pt(p):x,z=p;return ox+(x-31.5)*scale,oy+(79.5-z)*scale
    d.line([pt(p)for p in c['route']],fill='#e9b85c',width=4)
    for boss in ['imp','warden']:
        b=c[boss];lo,hi=b['min'],b['max'];l,_,_,bt=box(*lo);_,tp,rr,_=box(*hi);d.rectangle((l,tp,rr,bt),outline='#ff927c',width=4)
    labels=[('Sanctuary',c['extra']['recovery']),('Forge',c['workshop']),('Grove',c['grove']),('Supplies',c['provisions']),('Imp',c['imp']['spawn']),('Warden',c['warden']['spawn']),('Entry',c['extra']['entry']),('Waterworks',c['views']['waterworks'][:2])]
    for i,(name,p)in enumerate(labels,1):
        x,y=pt(p);d.ellipse((x-13,y-13,x+13,y+13),fill='#f5d494',outline='#15242b',width=2);d.text((x-6,y-13),str(i),font=font,fill='#15242b')
    d.text((80,1155),' → Exploration / circulation intention     □ Preserved encounter movement reserve',font=font,fill='#e9b85c')
    for i,(name,_)in enumerate(labels,1):d.text((80+((i-1)%4)*275,1200+((i-1)//4)*33),f'{i}. {name}',font=font,fill='#d9e4e8')
    im.save(path,optimize=True)

def main():
    p=argparse.ArgumentParser();p.add_argument('--config',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--diagram',type=Path);p.add_argument('--audit',type=Path);a=p.parse_args();c=json.loads(a.config.read_text());scene,info,walk,walls=compile_scene(c);a.output.mkdir(parents=True,exist_ok=True);f=a.output/'candidate.scene';f.write_text(scene);info['scene_sha256']=hashlib.sha256(f.read_bytes()).hexdigest();info['config_sha256']=hashlib.sha256(a.config.read_bytes()).hexdigest();(a.output/'composition.json').write_text(json.dumps(info,indent=2)+'\n')
    if a.diagram:
        if a.audit:
            audit=a.audit.read_text();assert 'issues=0' in audit;c=dict(c);c['route']=[list(map(int,line.split()[2:4]))for line in audit.splitlines()if line.startswith('PATH ')]
        a.diagram.parent.mkdir(parents=True,exist_ok=True);schematic(c,info,walk,walls,a.diagram)
    print(c['title'],info['chunks'],'chunks;',len(info['assets']),'placed assets;',info['water_tiles'],'real water tiles')
if __name__=='__main__':main()
