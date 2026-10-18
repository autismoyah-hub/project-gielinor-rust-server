"""Observe actual directed collision edges; never substitute a drawn walk mask."""
from pathlib import Path
import csv,json,sys
from collections import deque
T=Path(__file__).resolve().parent;S=T.parents[2];R=S.parent/'cinderworks-environment-private';P=S/'art/tower/cinderworks-environment'
def audit(name):
    root=R/name;plan=json.loads((P/'planning/floor-plan.json').read_text());edges={}
    for row in csv.DictReader((root/'walk-edges.tsv').open(),delimiter='\t'):
        x,z,dx,dz=[int(row[k])for k in ['x','z','dx','dz']];edges.setdefault((x,z),[]).append((x+dx,z+dz))
    def tree(start):
        parent={start:None};queue=deque([start])
        while queue:
            for q in edges.get(queue.popleft(),[]):
                if q not in parent:parent[q]=True;queue.append(q)
        return parent
    # Preserve actual parent edges for reproducible route, not a straight line.
    def path(start,goal):
        parent={start:None};queue=deque([start])
        while queue and goal not in parent:
            p=queue.popleft()
            for q in edges.get(p,[]):
                if q not in parent:parent[q]=p;queue.append(q)
        if goal not in parent:raise ValueError(('unreachable',start,goal))
        route=[goal]
        while route[-1]!=start:route.append(parent[route[-1]])
        return list(reversed(route))
    reachable=tree((56,56));rooms={};anchors={}
    for room in plan['rooms']:
        a,b,c,d=room['bounds'];cx,cz=room['center'];allowed=[p for p in reachable if a<=p[0]<=c and b<=p[1]<=d]
        assert allowed,room['id'];stand=min(allowed,key=lambda p:abs(p[0]-cx)+abs(p[1]-cz));rooms[room['id']]={'reachable_tiles':len(allowed),'review_standing_tile':stand,'path_from_arrival':path((56,56),stand)}
    objects=list(csv.DictReader((root/'objects.tsv').open(),delimiter='\t'))
    for anchor in plan['interaction_anchors']:
        x,z,l=anchor['local'];rows=[o for o in objects if int(o['x'])==x and int(o['z'])==z and int(o['shape'])in[0,4,10,11]]
        w=h=1
        if rows:
            obj=rows[-1];w,h=int(obj['width']),int(obj['length'])
            if int(obj['angle'])%2:w,h=h,w
        perimeter=set([(x-1,zz)for zz in range(z,z+h)]+[(x+w,zz)for zz in range(z,z+h)]+[(xx,z-1)for xx in range(x,x+w)]+[(xx,z+h)for xx in range(x,x+w)])
        if anchor['key'].startswith(('arrival','imp-spawn','warden-spawn')):perimeter.add((x,z))
        choices=[p for p in perimeter if p in reachable];assert choices,anchor['key']
        stand=min(choices,key=lambda p:len(path((56,56),p)))
        anchors[anchor['key']]={'target':anchor['local'],'standing_tile':stand,'path_from_arrival':path((56,56),stand),'existing_objects':rows,'scope':'Reachable standing tile only; live interaction must also be accepted by ordinary action dispatch.'}
    route=[];current=(56,56)
    for room_id in plan['traversal']:
        goal=tuple(rooms[room_id]['review_standing_tile']);segment=path(current,goal);route+=segment if not route else segment[1:];current=goal
    report={'authoritative_edges':str(root/'walk-edges.tsv'),'reachable_tiles':len(reachable),'all_rooms_accessible':True,'all_anchor_approaches_accessible':True,'rooms':rooms,'anchors':anchors,'whole_floor_route':route,'whole_floor_steps':len(route)-1,'scope':'Actual prepared server clipping. Does not prove live-client interaction, camera readability, visual quality, dynamic combat or completion rules.'}
    (root/'navigation.json').write_text(json.dumps(report,indent=2)+'\n');print('NAVIGATION',name,len(reachable),'reachable tiles;',len(rooms),'rooms;',len(anchors),'anchor approaches;',len(route)-1,'review walking steps')
    return report
if __name__=='__main__':audit(sys.argv[1]if len(sys.argv)>1 else'baseline-world')
