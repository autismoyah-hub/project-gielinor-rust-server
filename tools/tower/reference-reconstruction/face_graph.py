"""Explicit anatomical facial topology authored from the existing illustration.

Named cranial, brow, orbital, nasal, cheek and mandibular planes replace the
generic longitudinal head cage. No imported mesh or image billboard.
"""
import json
import bpy
from mathutils import Vector
from mesh_tools import ellipsoid
import surface as s
def create(mats):
    s.remove('^cranial_planes$|^angular_mandible$|^mouth_seam$|^orbital_recess_|^eye_iris_|^eye_pupil_|^nostril_')
    skin=mats['hide'].copy();skin['native_flat']=True
    v=[];names={};faces=[];material_indices=[]
    def point(name,p):names[name]=len(v);v.append(tuple(p))
    def face(seq,mat=0):faces.append(tuple(names[x]for x in seq));material_indices.append(mat)
    for name,p in {'crown':(0,-96,144),'forehead':(0,-116,138),'bridge_top':(0,-131,128),'bridge_low':(0,-143,118),'nose_tip':(0,-148,111),'mouth':(0,-145,104),'occiput':(0,-82,130),'throat':(0,-88,110)}.items():point(name,p)
    control={'temple':(22,-99,137),'forehead_side':(9,-108,141),'brow_crest':(15,-112,136),'brow_outer':(19,-117,132),'brow_inner':(8,-129,124.5),'orbit_outer_low':(20,-118,127),'orbit_lower':(16,-123,122),'orbit_inner_low':(9,-129,121),'cheek':(24,-114,119),'cheek_low':(18,-124,109),'jaw_corner':(18,-103,107),'nasal_edge':(8,-142,118),'nose_corner':(12,-147,112),'muzzle_corner':(14,-142,102),'mouth_end':(10,-145,105),'neck_side':(17,-84,120)}
    for side in [-1,1]:
        tag='p'if side>0 else'n'
        for name,(x,y,z)in control.items():point(name+tag,(side*x,y,z))
        def q(*seq,mat=0):
            resolved=[x if x in names else x+tag for x in seq]
            face(resolved if side>0 else list(reversed(resolved)),mat)
        q('crown','forehead','forehead_side')
        q('crown','forehead_side','temple')
        q('forehead_side','brow_crest','temple')
        q('brow_crest','brow_outer','temple')
        q('forehead','brow_inner','brow_crest')
        q('forehead','brow_crest','forehead_side')
        q('brow_crest','brow_inner','brow_outer')
        q('crown','temple','neck_side','occiput')
        q('forehead','bridge_top','brow_inner')
        q('bridge_top','brow_inner','orbit_inner_low','nasal_edge','bridge_low')
        q('brow_outer','temple','cheek','orbit_outer_low')
        q('orbit_outer_low','cheek','orbit_lower')
        q('orbit_inner_low','orbit_lower','cheek','cheek_low','muzzle_corner','nasal_edge')
        q('temple','neck_side','jaw_corner','cheek_low','cheek')
        q('cheek_low','jaw_corner','muzzle_corner')
        q('nasal_edge','muzzle_corner','nose_corner')
        q('bridge_low','nasal_edge','nose_corner','nose_tip')
        q('nose_tip','nose_corner','muzzle_corner','mouth_end','mouth')
        q('mouth','mouth_end','muzzle_corner','jaw_corner','throat')
        q('occiput','neck_side','jaw_corner','throat')
        # A closed five-sided orbital cup. Its rim is part of the head topology.
        ring=['brow_inner','brow_outer','orbit_outer_low','orbit_lower','orbit_inner_low'];center=Vector((side*14,-124,126))
        inner=[]
        for i,name in enumerate(ring):
            p=Vector(v[names[name+tag]]);p=center+(p-center)*.76+Vector((0,2,-.3));name2=f'cup_{tag}_{i}';point(name2,p);inner.append(name2)
        center_name='cup_center_'+tag;point(center_name,center+Vector((0,4,-.5)))
        for i in range(5):
            rr=[ring[i]+tag,ring[(i+1)%5]+tag,inner[(i+1)%5],inner[i]];face(rr if side>0 else list(reversed(rr)),0);face([inner[i],inner[(i+1)%5],center_name]if side>0 else[center_name,inner[(i+1)%5],inner[i]],1)
        ellipsoid('eye_iris_graph_'+tag,[side*14,-125,125.5],[2.3,2.1,2.2],mats['eye'],segments=12,rings=8)
        ellipsoid('eye_pupil_graph_'+tag,[side*14,-127,125.9],[.65,.65,1.2],mats['dark'],segments=8,rings=6)
        s.mesh('nostril_graph_'+tag,[(side*9,-146.5,117),(side*6,-148,116),(side*4,-149,112),(side*9,-148,114)],[(0,1,2,3)]if side>0 else[(3,2,1,0)],mats['dark'])
    o=s.mesh('reference_locked_cranial_topology',v,faces,skin);o.data.materials.append(mats['dark'])
    for p,mi in zip(o.data.polygons,material_indices):p.material_index=mi
    o['semantic_component']='Measured forehead, integrated orbital cups, nasal bridge, muzzle, zygomatic and mandibular planes';o['named_control_vertices']=json.dumps(names)
    jaw=mats['ventral'].copy();jaw['native_flat']=True
    s.loft('angular_mandible',[[-143,11,96,104.5],[-129,26,99,118],[-111,24,105,120],[-97,17,109,116]],jaw,section='cranium')
    s.loft('mouth_seam',[[-144,8.5,103,104],[-132,12,105.5,107]],mats['dark'],section='cranium')
    # The drawing's face is narrower than its neck/ears. Apply this local
    # anatomical correction only; measured horn and ear positions stay fixed.
    for obj in s.select('^reference_locked_cranial_topology$|^angular_mandible$|^mouth_seam$|^eye_.*_graph_|^nostril_graph_'):
        for vertex in obj.data.vertices:vertex.co.x*=.70
        obj.location.x*=.70
    return {'control_names':names,'vertices':len(v),'polygons':len(faces),'method':'Explicit independently authored facial surface graph and closed orbital cups'}
