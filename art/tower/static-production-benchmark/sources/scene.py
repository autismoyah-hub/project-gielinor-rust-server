from run import *
out=ROOT/'scene';out.mkdir(exist_ok=True)
run([BLENDER,'-b','--factory-startup','--python-exit-code','1','--python',HERE.parent/'high-fidelity/ground.py','--',out/'ground'],ROOT/'logs/ground.log')
rows=['light\t64\t768'];placements=[]
def add(label,path,x,z,y=0):rows.append('\t'.join(map(str,['owned',path,x,y,z,0,'-',0])));placements.append(dict(label=label,path=str(path),position=[x,y,z]))
add('ground',out/'ground/asset.model',0,0)
add('workstation',ROOT/'runs/workstation-final2/asset.model',-220,-130)
add('machine',ROOT/'runs/machine-final/asset.model',-195,110)
add('creature',ROOT/'runs/creature-final2/asset.model',135,-50,3)
add('arch',REPO/'art/tower/high-fidelity-mastery/assets/architecture/asset.model',0,250)
old=REPO/'art/tower/advanced-mastery/assets'
for x in [-265,265]:add('wall',old/'wall-bay/asset.model',x,285)
add('barrel',old/'supply-barrel/asset.model',300,210)
add('barrel2',old/'supply-barrel/asset.model',290,115)
manifest=out/'scene.tsv';manifest.write_text('\n'.join(rows)+'\n');(out/'placements.json').write_text(json.dumps(placements,indent=2))
cams={'overview':'look 850 -1050 -1250 0 -65 15 1750 1300 950','reverse-overview':'look -800 -1500 1000 0 -55 20 1650 1300 950','player-west':'player -70 -68 -20 1900 15300 430','player-east':'player 65 -60 -15 1950 1200 440','creature-close':'look 450 -240 -475 135 -62 -50 950 1100 850','machine-detail':'look 190 -290 -210 -195 -85 110 850 1100 850'}
for name,c in cams.items():
    camera=out/(name+'.camera');camera.write_text(c);target=out/'native'/name;run([NATIVE,CACHE,target,'scene',manifest,camera],target/'log.txt');Image.open(target/'scene.bmp').save(out/(name+'.png'))
print('SIX_STATIC_SCENE_VIEWS')
