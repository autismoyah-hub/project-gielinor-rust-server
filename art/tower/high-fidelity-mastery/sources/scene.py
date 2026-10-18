"""Original integrated workshop; fixed native views and reusable Blender scene."""
from review import *
out=ROOT/'scene';out.mkdir(exist_ok=True)
run([BLENDER,'-b','--factory-startup','--python-exit-code','1','--python',HERE/'ground.py','--',out/'ground'],ROOT/'logs/ground.log')
rows=['light\t64\t768'];placements=[]
def add(label,path,x,z,q=0,y=0):
    rows.append('\t'.join(map(str,['owned',path,x,y,z,q,'-',0])));placements.append(dict(label=label,path=str(path),position=[x,y,z],quarter_turns=q))
add('Yard terrain',out/'ground/asset.model',0,0)
add('Refined arch',ROOT/'runs/architecture-revision1/asset.model',0,220)
add('Refined pump',ROOT/'runs/machine-revision1/asset.model',-155,-25)
add('Cindermaw',ROOT/'runs/creature-final/asset.model',125,-80)
old=REPO/'art/tower/advanced-mastery/assets'
for x in [-265,265]:add('Retained original wall',old/'wall-bay/asset.model',x,265)
add('Retained barrel',old/'supply-barrel/asset.model',270,180)
add('Retained stock rack',old/'stock-rack/asset.model',-265,170)
scene=out/'scene.tsv';scene.write_text('\n'.join(rows)+'\n');(out/'placements.json').write_text(json.dumps(placements,indent=2))
cams={'overview':'look 780 -900 -1070 0 -65 20 1080 1200 900','player-west':'player -90 -65 -15 1850 15300 390','player-east':'player 70 -65 -30 1900 1200 390','creature-close':'look 380 -245 -455 125 -67 -80 880 1100 800'}
for name,c in cams.items():
    camera=out/(name+'.camera');camera.write_text(c);target=out/'native'/name;run([NATIVE,CACHE,target,'scene',scene,camera],target/'log.txt');Image.open(target/'scene.bmp').save(out/(name+'.png'))
print('YARD_NATIVE_VIEWS_COMPLETE')
