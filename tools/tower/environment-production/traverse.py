"""Actual client/server walking and captures, with no position injection."""
from pathlib import Path
import json,time,hashlib,sys,shutil
from collections import deque
from PIL import Image
from native_session import R,command
from navigation import audit
def wait_position(session,target,seconds=14):
    before=time.monotonic();seen=[]
    while time.monotonic()-before<seconds:
        value=command(session,'observe');pos=tuple(value['position'][1:]);seen.append({'position':pos,'tick':value['tick'],'sequence':value['sequence']})
        if pos==tuple(target):return value,seen
        time.sleep(.3)
    raise TimeoutError((target,seen[-5:]))
def run():
    info=json.loads((R/'latest-native-session.json').read_text());session=Path(info['session']);nav=json.loads((R/'07-whole-floor-refinement/navigation.json').read_text());import csv
    edges={}
    for row in csv.DictReader((R/'07-whole-floor-refinement/walk-edges.tsv').open(),delimiter='\t'):
        x,z,dx,dz=[int(row[k])for k in ['x','z','dx','dz']];edges.setdefault((10368+x,64+z),[]).append((10368+x+dx,64+z+dz))
    def path(start,goal,avoid):
        parent={start:None};queue=deque([start])
        while queue and goal not in parent:
            p=queue.popleft()
            for q in edges.get(p,[]):
                if q not in parent and q not in avoid:parent[q]=p;queue.append(q)
        if goal not in parent:raise RuntimeError(('live review route unavailable',start,goal))
        result=[goal]
        while result[-1]!=start:result.append(parent[result[-1]])
        return list(reversed(result))
    out=session/'traversal';out.mkdir(exist_ok=True);report={'session':str(session),'method':'Ordinary native Action::Walk through current client policy and authoritative server collision, with observed positions and actual client frame captures. No camera-only navigation or teleportation.','started_unix':time.time(),'rooms':[],'segments':[],'failures':[],'scope':'Environment navigation. Does not claim service completion, boss victory, reward eligibility or full gameplay-loop validation.'}
    value=command(session,'observe');assert value['ready']and value['position'][1]>10000
    overrides={'G':[58,70],'F3':[72,70],'F2':[38,42]}
    try:
        for room_id in ['A','B','C','D1','A','D2','F1','S','F2','A','F3','G','H1','H2','A']:
            local=overrides.get(room_id,nav['rooms'][room_id]['review_standing_tile']);goal=(10368+local[0],64+local[1]);current=tuple(value['position'][1:]);attempts=0
            while current!=goal:
                avoid=set()
                for npc in value.get('npcs',[]):
                    size=2 if npc['id']==16295 else 1
                    for dx in range(size):
                        for dz in range(size):avoid.add((npc['x']+dx,npc['z']+dz))
                avoid.discard(goal);route=path(current,goal,avoid);destination=route[min(6,len(route)-1)]
                before=time.monotonic();receipt=command(session,f'walk {destination[0]} {destination[1]}');assert receipt['status']=='accepted'
                try:value,seen=wait_position(session,destination)
                except TimeoutError as error:
                    report['failures'].append({'room':room_id,'destination':destination,'error':str(error)});attempts+=1
                    if attempts>=3:raise
                    value=command(session,'observe');current=tuple(value['position'][1:]);continue
                report['segments'].append({'room':room_id,'from':current,'destination':destination,'elapsed_seconds':time.monotonic()-before,'observed':seen});current=tuple(value['position'][1:])
                (out/'progress.json').write_text(json.dumps(report,indent=2))
            yaw=8192 if room_id=='H1'else 12288 if room_id=='H2'else 0
            command(session,f'camera 2000 {yaw}');value=command(session,'capture');frame=session/f'frame-{value["sequence"]}.bmp'
            for _ in range(80):
                if frame.exists():break
                time.sleep(.05)
            assert frame.exists();name=f'{len(report["rooms"]):02d}-{room_id}.png';Image.open(frame).save(out/name)
            report['rooms'].append({'id':room_id,'standing_world':value['position'],'tick':value['tick'],'sequence':value['sequence'],'frame':name,'frame_sha256':hashlib.sha256(frame.read_bytes()).hexdigest(),'camera_pitch':2000,'camera_yaw':yaw})
            (out/'progress.json').write_text(json.dumps(report,indent=2));print('NATIVE_ROOM',room_id,value['position'],'tick',value['tick'],'segments',len(report['segments']),flush=True)
        report['finished_unix']=time.time();report['all_review_rooms_traversed']=True;report['arrived_back_at_sanctuary']=True
    finally:(out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
    return report
if __name__=='__main__':run()
