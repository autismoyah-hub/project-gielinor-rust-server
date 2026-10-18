from run import *
scene=ROOT/'scene/scene.tsv'
scene.write_text(scene.read_text().replace(str(ROOT/'scene/ground/asset.model'),str(ROOT/'scene/ground-v2/asset.model')))
cams={'overview':'look 790 -880 -1050 0 -65 20 1000 1200 900','player-west':'player -90 -65 -15 1850 15300 380','player-east':'player 70 -65 -30 1900 1200 380','creature-detail':'look 390 -220 -440 130 -60 -80 850 1000 800','ground-fix-same-camera':'player -45 -72 -30 2000 15000 490'}
for name,cam in cams.items():
    path=scene.parent/(name+'.camera');path.write_text(cam);dest=scene.parent/'fixed-views'/name;run([NATIVE,CACHE,dest,'scene',scene,path],dest/'render.log');Image.open(dest/'scene.bmp').save(scene.parent/(name+'.png'))
