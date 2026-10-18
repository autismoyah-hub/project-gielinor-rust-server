"""Reference-locked original Ashhorn surfaces, built around named measurements.

No imported creature mesh. Static landmark cages, independently authored
anatomical surfaces, local sculpting, true solid armor and native face colors.
"""
from pathlib import Path
import sys,json,math,colorsys
import bpy,bmesh
from mathutils import Vector
T=Path(__file__).resolve().parent;sys.path.insert(0,str(T.parent/'constrained'));sys.path.insert(0,str(T.parent/'mesh-study'))
sys.path.insert(0,str(T))
import surface as s
from mesh_tools import ellipsoid
R=T.parents[3]/'ashhorn-reference-private'
PALETTE={'hide':(.37,.36,.34),'ventral':(.54,.43,.30),'plate':(.42,.27,.16),'edge':(.65,.43,.23),'horn':(.67,.59,.44),'claw':(.74,.68,.52),'dark':(.12,.11,.095),'eye':(.95,.52,.08)}
def native(rgb):
    h,l,ss=colorsys.rgb_to_hls(*rgb);return (int(h*64)<<10)|(min(7,int(ss*8))<<7)|max(2,min(126,int(l*127)))
def profile(name,rows,mat,shape=None):
    shape=shape or [(0,1),(.5,.98),(.88,.85),(1,.6),(.88,.3),(.48,.06),(0,0),(-.48,.06),(-.88,.3),(-1,.6),(-.88,.85),(-.5,.98)]
    n=len(shape);v=[]
    for row in rows:
        y,w,low,high=row[:4];cx=row[4] if len(row)>4 else 0
        v.extend([(cx+a*w,y,low+b*(high-low))for a,b in shape])
    faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
    faces.extend((i*n+j,i*n+(j+1)%n,(i+1)*n+(j+1)%n,(i+1)*n+j)for i in range(len(rows)-1)for j in range(n))
    obj=s.mesh(name,v,faces,mat);obj['semantic_component']=name;obj['station_controls']=json.dumps(rows);return obj
def limb(name,points,radii,mat):
    obj=s.landmark_limb(name,points,radii,mat);obj['distal_ring_indices']=list(range(len(obj.data.vertices)-10,len(obj.data.vertices)));return obj
def curved(name,points,radii,mat,sides=10):
    o=s.tube(name,points,radii,mat,sides=sides);o['semantic_component']=name;return o
def skull(rows,mat):
    vertices=[]
    for y,w,low,top,side in rows:
        half=[(0,top),(.43*w,top-1),(.78*w,top-5),(w,side),(.86*w,low+4),(.43*w,low),(0,low-1)]
        ring=half+[(-x,z)for x,z in half[-2:0:-1]]
        vertices.extend((x,y,z)for x,z in ring)
    n=12;faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
    faces.extend((i*n+j,i*n+(j+1)%n,(i+1)*n+(j+1)%n,(i+1)*n+j)for i in range(len(rows)-1)for j in range(n))
    o=s.mesh('cranial_planes',vertices,faces,mat);o['semantic_component']='skull nasal bridge brow and zygomatic planes';o['section_landmarks']=json.dumps(rows);return o
def add_face(P,mats,phase):
    rows=[[-143,11.5,110,119,114],[-134,15,110,123,116],[-125,17.5,111,128,120],[-116,21,111,136,122],[-106,25,113,142,126],[-94,24,113,141,129],[-83,18,110,134,124]]
    headmat=mats['hide'];jawmat=mats['ventral']
    if phase>=2:
        headmat=headmat.copy();headmat['native_flat']=True;jawmat=jawmat.copy();jawmat['native_flat']=True
        rows[3][1]=19.5;rows[4][1]=22.5;rows[5][1]=21.5
    if phase>=3:rows[0]=[-141,13.5,110,119,114];rows[1]=[-132,16.5,110,123,116]
    if phase>=6:
        rows[0][2]=105;rows[1][2]=107
    head=skull(rows,headmat)
    jawrows=[[-142,9.5,103,108],[-132,13,100,109],[-119,16,99,111],[-106,18,102,115],[-94,14,110,120]]
    if phase>=6:jawrows=[[-141,7.5,98,103.5],[-132,12.5,99,106],[-119,16,100,111],[-106,18,104,116],[-94,14,111,120]]
    profile('angular_mandible',jawrows,jawmat)
    profile('mouth_seam',[[-141,10,104,105.5],[-130,13,105,107]]if phase>=6 else[[-141,10,107,111],[-125,14,107,112],[-106,16,111,115]],mats['dark'])
    for side in [-1,1]:
        pos=P['eye_pos'if side>0 else'eye_neg'];cx=side*(abs(pos[0])+3.5);cy=pos[1]-1;cz=pos[2]
        if phase>=2:
            cutter=ellipsoid('Temporary orbital excavation',[cx,cy,cz+1],[6.3,6.5,3.5 if phase>=3 else 5.4],mats['dark'],segments=12,rings=8)
            mod=head.modifiers.new('Integrated orbital recess','BOOLEAN');mod.operation='DIFFERENCE';mod.solver='EXACT';mod.object=cutter;s.apply(head,mod);bpy.data.objects.remove(cutter,do_unlink=True)
            s.masked_deform([head],[side*13,-114,132],[10,10,7],delta=[0,-1,1.2])
        ellipsoid(f'orbital_recess_{side}',[cx,cy+1,cz],[5.1,4.9,2.7 if phase>=3 else 4.0],mats['dark'],segments=12,rings=8)
        ellipsoid(f'eye_iris_{side}',[cx,cy-1.7,cz+.1],[3.0,2.8,2.0 if phase>=3 else 3.0],mats['eye'],segments=10,rings=6)
        ellipsoid(f'eye_pupil_{side}',[cx+side*.5,cy-4,cz+.2],[.65,.7,1.2],mats['dark'],segments=8,rings=6)
        if phase>=3:
            points=[(side*6,-127,130),(side*20,-122,134),(side*24,-111,136),(side*8,-111,137),(side*7,-126,127),(side*21,-121,130),(side*25,-112,131),(side*8,-113,133)]
            ridge=s.mesh('Temporary integrated supraorbital ridge',points,[(0,1,2,3),(4,7,6,5),(0,4,5,1),(1,5,6,2),(2,6,7,3),(3,7,4,0)],headmat)
            mod=head.modifiers.new('Continuous heavy brow anatomy','BOOLEAN');mod.operation='UNION';mod.solver='EXACT';mod.object=ridge;s.apply(head,mod);bpy.data.objects.remove(ridge,do_unlink=True)
        if phase>=6:
            v=[(side*13,-127,116),(side*22,-119,118),(side*26,-105,124),(side*17,-103,120),(side*12,-126,112),(side*21,-116,110),(side*23,-104,115),(side*15,-104,114)]
            cheek=s.mesh('Temporary connected zygomatic keel',v,[(0,1,2,3),(4,7,6,5),(0,4,5,1),(1,5,6,2),(2,6,7,3),(3,7,4,0)],headmat);mod=head.modifiers.new('Connected cheekbone anatomy','BOOLEAN');mod.operation='UNION';mod.solver='EXACT';mod.object=cheek;s.apply(head,mod);bpy.data.objects.remove(cheek,do_unlink=True)
        # Folded ear shell: outer cartilage and inset, with a real ridge.
        base=P['ear_pos'if side>0 else'ear_neg'];x=side*(abs(base[0])+4)
        v=[(x,-94,133),(x+side*11,-73,159),(x+side*9,-67,143),(x,-83,129),(x+side*2,-81,140),(x-side*3,-88,135)]
        o=s.mesh(f'folded_ear_{side}',v,[(0,1,4),(1,2,4),(2,3,4),(3,0,4),(0,5,1),(1,5,2),(2,5,3),(3,5,0)],mats['hide']);o.data.materials.append(mats['ventral'])
        for poly in o.data.polygons:
            if poly.index in [0,1]:poly.material_index=1
        s.shell_profile(f'nostril_{side}',[(side*8,116),(side*4,116),(side*3,112)],1.6,mats['dark'],axis='Y',offset=(0,-143,0))
    root=P['horn_root_pos'];tip=P['horn_tip_pos'];root=[max(12,root[0]),root[1],root[2]-2]
    points=[root,[root[0]+4,root[1]+8,root[2]+11],[23,-80,157],[29,-64,167],tip]
    horn=curved('long_swept_horn',points,[10.5,9,7,4,.35],mats['horn'],12)
    # Additional station density makes the three bands geometrically distinct.
    if phase>=1:
        s.remove('^long_swept_horn$');points=[];radii=[]
        controls=[(Vector(p),r)for p,r in zip([root,[root[0]+4,root[1]+8,root[2]+11],[23,-80,157],[29,-64,167],tip],[10.5,9,7,4,.35])]
        for i in range(len(controls)-1):
            for f in [0,.34,.67]:points.append(tuple(controls[i][0].lerp(controls[i+1][0],f)));radii.append(controls[i][1]*(1-f)+controls[i+1][1]*f)
        points.append(tip);radii.append(.35);horn=curved('long_swept_horn',points,radii,mats['horn'],12)
        horn.data.materials.append(mats['dark'])
        for f in horn.data.polygons:
            if f.index>=2 and (f.index-2)//12 in [1,4,8]:f.material_index=1
    shortroot=P['horn_root_neg'];shorttip=P['horn_break_neg'];short=curved('broken_horn',[shortroot,[(shortroot[0]+shorttip[0])/2,(shortroot[1]+shorttip[1])/2,145],shorttip],[9.5,8.8,7],mats['horn'],11)
    # Authored uneven fracture cap, not a clean cylinder top.
    t=Vector(shorttip);tangent=(t-Vector(shortroot)).normalized();indices=list(short.data.polygons[1].vertices);jitter=[0,2,1,-1,2,0,1,2,-1,1,0]
    for i,index in enumerate(indices):short.data.vertices[index].co+=tangent*jitter[i]
    ring=[tuple(short.data.vertices[i].co+tangent*.3)for i in indices];ring.append(tuple(t-tangent))
    short.data.materials.append(mats['dark']);short.data.polygons[1].material_index=1
    s.mesh('jagged_fracture',ring,[(i,(i+1)%11,11)for i in range(11)],mats['dark'])
    return head
def add_paw(side,kind,point,mats,phase):
    x,y,z=point;width=20 if kind=='fore' else 17
    profile(f'{kind}_paw_{side}',[[y-9,width*.82,1,14,x],[y+1,width,0,18,x],[y+15,width*.63,3,19,x]],mats['hide'])
    spacing=11.5 if kind=='fore' else 10
    for digit in [-1,0,1]:
        dx=x+digit*spacing
        profile(f'{kind}_toe_{side}_{digit}',[[y-17,4.7,1,8,dx],[y-10,6.0,0,13,dx],[y+2,5.5,3,16,dx]],mats['hide'])
        profile(f'{kind}_claw_{side}_{digit}',[[y-12,3.5,4,10,dx],[y-17,3.7,1,8,dx],[y-23,1.5,0,3,dx],[y-24,.3,0,.7,dx]],mats['claw'],shape=[(0,1),(1,.65),(.85,0),(-.85,0),(-1,.65)])
def armor(P,mats,phase):
    body=s.select('^continuous_anatomy$')[0];inverse=body.matrix_world.inverted()
    faces=[(i*3+j,i*3+j+1,(i+1)*3+j+1,(i+1)*3+j)for i in range(2)for j in range(2)]
    def shell(name,points,fs,thickness=3):
        obj=s.mesh(name,points,fs,mats['plate']);obj.data.materials.append(mats['edge']);mod=obj.modifiers.new('Solid grown plate with worn rim','SOLIDIFY');mod.thickness=thickness;mod.offset=0;mod.material_offset_rim=1;s.apply(obj,mod);obj['semantic_component']=name;obj['control_cage']=json.dumps(points)
    for i in range(5):
        ridge=P[f'armor_{i}_ridge'];y=ridge[1];width=[44,40,34,29,25][i];points=[]
        for dy,sc in [(-13,.72),(0,1),(16,.62)]:
            for col in [-1,0,1]:
                x=col*width*sc;depth=y+dy+(3 if col==0 and dy>0 else -2 if col==0 and dy<0 else 0)
                hit,p,_,_=body.ray_cast(inverse@Vector((x,depth,400)),Vector((0,0,-1)))
                if not hit:
                    # Overhanging shoulder shield uses its measured crest.
                    z=ridge[2]-7-abs(col)*8
                else:
                    h=(body.matrix_world@p).z;z=h+4+max(0,ridge[2]-h-4)*(1-abs(col)*.7)
                    if phase>=6:z=max(h+3.5,ridge[2]+(-4 if dy<0 else -9 if dy>0 else 0)-abs(col)*12)
                points.append((x,depth,z))
        shell(f'dorsal_shingle_{i}',points,faces)
    for side in [-1,1]:
        controls=[('upper',[
           [(27,-69,135),(43,-69,129),(51,-69,116)],
           [(30,-52,145),(53,-52,132),(64,-52,111)],
           [(33,-28,136),(59,-28,117),(64,-28,105)]]),('lower',[
           [(45,-59,120),(57,-59,111),(64,-59,99)],
           [(49,-44,122),(64,-44,106),(68,-44,88)],
           [(48,-25,115),(63,-25,100),(63,-25,86)]])]
        if phase>=2:
            controls=[('upper',[
             [(26,-76,139),(41,-70,143),(55,-57,135)],
             [(31,-75,130),(47,-67,131),(64,-48,123)],
             [(39,-70,122),(55,-60,116),(65,-40,113)]]),('lower',[
             [(36,-72,126),(50,-71,122),(63,-56,116)],
             [(38,-68,114),(55,-65,108),(68,-47,103)],
             [(42,-62,101),(58,-57,94),(66,-43,91)]])]
        for name,rows in controls:
            points=[]
            for row in rows:
                for x,y,z in row:
                    if phase>=6:
                        y+=15;z=(128+(z-128)*.5)if name=='upper'else(108+(z-108)*.65)
                    hit,p,_,_=body.ray_cast(inverse@Vector((side*250,y,z)),Vector((-side,0,0)))
                    if hit:x=max(x,abs((body.matrix_world@p).x)+3.0)
                    points.append((side*x,y,z))
            shell(f'shoulder_shingle_{name}_{side}',points,faces,3.0)
def smooth_path(points,radii,steps=4):
    pp=list(map(Vector,points));result=[];rr=[]
    for i in range(len(pp)-1):
        a,b,c,d=pp[max(i-1,0)],pp[i],pp[i+1],pp[min(i+2,len(pp)-1)]
        for j in range(steps):
            t=j/steps;result.append(tuple(.5*((2*b)+(-a+c)*t+(2*a-5*b+4*c-d)*t*t+(-a+3*b-3*c+d)*t*t*t)));rr.append(radii[i]*(1-t)+radii[i+1]*t)
    result.append(tuple(pp[-1]));rr.append(radii[-1]);return result,rr
def build(C,phase):
    P=C['landmarks'];mats={k:s.native_color(k,native(v))for k,v in PALETTE.items()}
    bodyrows=[[-83,24,89,133],[-66,35,72,137],[-49,44,58,139],[-31,48,51,136],[-10,43,58,129],[15,35,65,120],[36,32,65,112],[59,36,65,107],[78,39,69,104],[97,28,79,99],[111,12,85,94]]
    profile('skin_ribcage_waist_pelvis',bodyrows,mats['hide'])
    profile('skin_throat_neck',[[-103,19,111,139],[-88,26,93,140],[-69,32,77,140],[-47,36,69,137]],mats['hide'])
    for side,key in [(1,'pos'),(-1,'neg')]:
        shoulder,elbow,wrist,paw=[P[n+'_'+key]for n in ['shoulder','elbow','wrist','forepaw']]
        points=[[side*33,-47,117],[shoulder[0],shoulder[1],101],elbow,[(elbow[0]+wrist[0])/2,(elbow[1]+wrist[1])/2,38],wrist,[paw[0],paw[1]+5,12]]
        radii=[[20,23],[22,21],[15,16],[12,13],[9,11],[11,12]] if phase<2 else [[20,23],[21,20],[11,12],[15,14],[8.5,10],[11,12]]
        limb('skin_fore_'+key,points,radii,mats['hide'])
        hip,knee,hock,paw=[P[n+'_'+key]for n in ['hip','knee','hock','hindpaw']]
        points=[[side*29,70,104],hip,knee,[(knee[0]+hock[0])/2,(knee[1]+hock[1])/2,45],hock,[paw[0],paw[1]+6,13]]
        limb('skin_hind_'+key,points,[[21,25],[23,25],[17,19],[11,13],[8,9],[9,11]],mats['hide'])
        add_paw(side,'fore',P['forepaw_'+key],mats,phase);add_paw(side,'hind',P['hindpaw_'+key],mats,phase)
    if phase>=1:
        s.execute({'materials':{'hide':native(PALETTE['hide'])},'operations':[{'op':'fuse','target':'^skin_','name':'continuous_anatomy','material':'hide','voxel':1.25,'triangles':3700}]})
        body=s.select('^continuous_anatomy$')[0]
        s.surface_regions(body,[(mats['ventral'],lambda x,y,z:abs(x)<31 and ((y<-30 and 55<z<121)or(-30<=y<83 and z<86)))])
    head=add_face(P,mats,phase)
    if phase>=8:
        import face_graph
        face_graph.create(mats)
    root=P['tail_root'];mid=P['tail_mid'];tip=P['tail_tip'];points=[root,[0,125,76],mid,[tip[0]*.7,190,80],tip];radii=[11,9,7,4.5,.9]
    if phase>=1:points,radii=smooth_path(points,radii)
    curved('sweeping_tail',points,radii,mats['hide'],12)
    if phase>=1:armor(P,mats,phase)
    return {'phase':phase,'body_stations':bodyrows,'palette_rgb':PALETTE,'named_landmarks':P,'construction':'Independent profile surfaces, anatomical chains, local fusion, sculpted skull, true toes/claws, folded ears and asymmetric swept horns','materials':{k:native(v)for k,v in PALETTE.items()},'operations':[]}
if __name__=='__main__':
    config,out,phase=sys.argv[sys.argv.index('--')+1:];C=json.loads(Path(config).read_text());out=Path(out);bpy.ops.wm.open_mainfile(filepath=str((R/'reference-workspace.blend').resolve()))
    recipe=build(C,int(phase));paint_report=None
    if int(phase)>=4:
        import paint
        if int(phase)>=5:C['paint_semantic_constraints']=True
        paint_report=paint.apply(C,R/'references')
    s.save(out,recipe,[{'framework_source':'reference-workspace.blend','constraints_file':str(Path(config).name),'stage':int(phase)}]);(out/'constraints.json').write_text(json.dumps(C,indent=2));(out/'build-snapshot.py').write_text(Path(__file__).read_text())
    if paint_report:(out/'palette-guidance.json').write_text(json.dumps(paint_report,indent=2));(out/'paint-snapshot.py').write_text((T/'paint.py').read_text())
    if int(phase)>=8:(out/'face-graph-snapshot.py').write_text((T/'face_graph.py').read_text())
