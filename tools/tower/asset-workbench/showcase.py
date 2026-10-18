"""Render an isolated mixed Cinderworks vignette through the native Rust client.

Inputs are owned authoring outputs and read-only cache scenery IDs. Shared scene
images are protected local evidence, never automatically published with assets.
"""
import argparse,json,subprocess,math
from pathlib import Path
from PIL import Image,ImageDraw

VIEWS={
 'overview':'look 940 -850 -1100 0 -65 30 1350 1440 960',
 'reverse':'look -840 -730 1040 0 -65 20 1180 1280 900',
 'player-entrance':'player 0 -55 -15 2560 1300 1120',
 'player-workbench':'player 240 -80 150 2100 14000 300',
 'player-creature':'player 180 -36 -150 2100 900 250',
}

def run(a):
    a.out.mkdir(parents=True,exist_ok=True);assets={k:Path(v).resolve()for k,v in json.loads(a.assets.read_text()).items()};rows=[['light','64','768']]
    for key,xyz in [('showcase-terrain',[0,0,0]),('sluice-keeper-arcade',[0,0,145]),('counterweight-hoist',[240,0,150]),('kilnback-rummager',[180,0,-150])]:
        root=assets[key];rows.append(['owned',str(root/'asset.model'),*map(str,xyz),'0',str(root/'idle')if key=='kilnback-rummager'else'-','0'])
    # A small, explicit amount of genuine scenery; no map or definition writes.
    rows += [['object','6189','-270','0','145','0'],['object','362','-285','0','-140','0'],['object','325','-355','0','225','0'],['npc','3106','-195','0','-80','0']]
    for x in [-320,-192,192,320]:rows.append(['object','6143',str(x),'0','345','1'])
    manifest=a.out/'scene.tsv';manifest.write_text('\n'.join('\t'.join(row)for row in rows)+'\n')
    for name,text in VIEWS.items():
        camera=a.out/(name+'.camera');camera.write_text(text);subprocess.run([str(a.native),str(a.cache),str(a.out/'views'/name),'scene',str(manifest),str(camera)],check=True)
    actor_index=next(i for i,row in enumerate(rows)if row[0]=='owned'and row[1]==str(assets['kilnback-rummager']/'asset.model'));recipe=json.loads((assets['kilnback-rummager']/'recipe.json').read_text());gait=recipe['locomotion'];speed=gait['stride']/(gait['stance_fraction']*recipe['actions']['walk']['duration_ticks']);motion=[]
    if not a.stills_only:
        camera=a.out/'player-creature.camera'
        for action in ['walk','attack','death']:
            active=[list(r)for r in rows];active[actor_index][6]=str(assets['kilnback-rummager']/action);active[actor_index][7]=str(-speed if action=='walk'else 0);mf=a.out/(action+'-scene.tsv');mf.write_text('\n'.join('\t'.join(row)for row in active)+'\n');duration=recipe['actions'][action]['duration_ticks'];ticks=list(range(0,duration+1,3))
            if duration not in ticks:ticks.append(duration)
            if action=='death':ticks.append(duration+recipe['actions'][action].get('terminal_pose_hold',0))
            frames=[]
            for tick in ticks:
                out=a.out/'motion'/action/f'tick-{tick:04}';subprocess.run([str(a.native),str(a.cache),str(out),'scene',str(mf),str(camera),str(tick)],check=True);im=Image.open(out/'scene.bmp').convert('RGB');ImageDraw.Draw(im).text((14,14),f'{action} | native tick {tick}',fill='white');frames.append(im)
            durations=[max(20,(ticks[i+1]-t)*20)if i+1<len(ticks)else 800 for i,t in enumerate(ticks)];frames[0].save(a.out/(action+'-scene.gif'),save_all=True,append_images=frames[1:],duration=durations,loop=0);motion.append(dict(action=action,native_frames=len(ticks),ticks=ticks,world_travel_z_per_tick=-speed if action=='walk'else 0))
    (a.out/'showcase-proof.json').write_text(json.dumps(dict(original_assets=list(assets),authentic_object_ids=[6189,362,325,6143],authentic_npc_ids=[3106],views=list(VIEWS),motion=motion,live_map_changed=False,cache_changed=False,artistic_acceptance=False),indent=2));print('NATIVE_MIXED_SHOWCASE_COMPLETE')
if __name__=='__main__':
    p=argparse.ArgumentParser()
    for name in ['native','cache','assets','out']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--stills-only',action='store_true');run(p.parse_args())
