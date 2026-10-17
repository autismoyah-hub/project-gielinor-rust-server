"""Original Cinderworks geometry informed by measured native reference studies.

No cache meshes, texture samples, Blender reference files or extracted arrays
are read. Reference IDs in manifests describe analysis, never copied inputs.
"""
import argparse,bpy,bmesh,json,math,sys,random,colorsys
from pathlib import Path
from mathutils import Vector

# Authored packed-HSL targets translated to glTF linear RGB, then independently
# quantized by the converter. Native lighting remains entirely in Rust.
COLORS={'stone':(6,0,30),'stone_light':(6,0,35),'stone_dark':(6,0,25),'cap':(6,0,38),'mortar':(6,1,20),'iron':(0,0,25),'iron_light':(0,0,34),'soot':(0,0,16),'wood':(6,3,32),'wood_light':(6,3,38),'brass':(9,3,48),'copper':(4,3,37),'heat':(3,6,62),'hot':(7,6,79),'water':(30,3,35),'damp':(25,2,28),'soil':(6,2,29),'moss':(16,2,26),'cloth':(27,3,39),'paper':(8,2,68)}

def main():
 p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--iteration',type=int,default=1);a=p.parse_args(sys.argv[sys.argv.index('--')+1:]);a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=True)
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
 mats={};pieces=[];assets=[]
 for name,(h,s,l) in COLORS.items():
  rgb=colorsys.hls_to_rgb((h+.25)/64,(l+.25)/128,(s+.25)/8)
  rgb=[c/12.92 if c<=.04045 else ((c+.055)/1.055)**2.4 for c in rgb]
  for smooth in [False,True]:
   m=bpy.data.materials.new(name+('-smooth' if smooth else ''));m.diffuse_color=(*rgb,1);m.use_nodes=True;m.use_backface_culling=True;m['native_render_type']=0 if smooth else 1
   m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=(*rgb,1);mats[name,smooth]=m
 def mesh(name,verts,faces,material='stone',smooth=False,face_mats=None):
  me=bpy.data.meshes.new(name);me.from_pydata(verts,[],faces);me.update();o=bpy.data.objects.new(name,me);bpy.context.collection.objects.link(o)
  names=list(dict.fromkeys(face_mats or [material]));[me.materials.append(mats[n,smooth]) for n in names]
  for f,n in zip(me.polygons,face_mats or [material]*len(faces)):f.material_index=names.index(n)
  bm=bmesh.new();bm.from_mesh(me);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bmesh.ops.triangulate(bm,faces=list(bm.faces));bm.to_mesh(me);bm.free();pieces.append(o);return o
 def box(pos,size,material='stone',angle=0):
  x,y,z=pos;dx,dy,dz=[s/2 for s in size];c,s=math.cos(angle),math.sin(angle)
  verts=[(x+u*c-v*s,y+u*s+v*c,z+w) for w in [-dz,dz] for u,v in [(-dx,-dy),(dx,-dy),(dx,dy),(-dx,dy)]]
  return mesh('block',verts,[(0,3,2,1),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)],material)
 def extrusion(poly,depth,material='stone',y=0,face_mats=None):
  n=len(poly);verts=[(x,v,z) for v in [y-depth/2,y+depth/2] for x,z in poly];faces=[tuple(range(n-1,-1,-1)),tuple(range(n,2*n))]+[(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
  return mesh('profile',verts,faces,material,face_mats=face_mats)
 def lathe(pos,rings,material='stone',sides=8,phase=0,smooth=False):
  x,y,z=pos;verts=[(x+r*math.cos(i*math.tau/sides+phase+turn),y+r*math.sin(i*math.tau/sides+phase+turn),z+h) for h,r,turn in rings for i in range(sides)];faces=[tuple(range(sides-1,-1,-1))]
  for j in range(len(rings)-1):
   for i in range(sides):k=j*sides+i;n=j*sides+(i+1)%sides;faces.append((k,n,n+sides,k+sides))
  faces.append(tuple(range((len(rings)-1)*sides,len(rings)*sides)));return mesh('shaped section',verts,faces,material,smooth)
 def cylinder(pos,radius,length,material='iron',axis=(0,0,1),sides=8,radius2=None,smooth=False):
  o=lathe((0,0,0),[(-length/2,radius,0),(length/2,radius if radius2 is None else radius2,0)],material,sides,smooth=smooth);o.location=pos;o.rotation_euler=Vector(axis).to_track_quat('Z','Y').to_euler();return o
 def beam(start,end,width,material='wood'):
  v=Vector(end)-Vector(start);o=box((0,0,0),(width,width,v.length),material);o.location=(Vector(start)+Vector(end))/2;o.rotation_euler=v.to_track_quat('Z','Y').to_euler();return o
 def asset(name,index,footprint,build,purpose,reference,collision):
  nonlocal pieces
  pieces=[];bpy.ops.object.select_all(action='DESELECT');build()
  for o in pieces:o.select_set(True)
  bpy.context.view_layer.objects.active=pieces[0];bpy.ops.object.join();o=bpy.context.object;o.name=name
  bpy.ops.object.transform_apply(location=False,rotation=True,scale=True)
  for v in o.data.vertices:v.co+=o.location
  o.location=(0,0,0)
  bpy.ops.export_scene.gltf(filepath=str(a.output/(name+'.glb')),export_format='GLB',use_selection=True,export_apply=True,export_yup=True,export_materials='EXPORT',export_extras=True,export_cameras=False,export_lights=False)
  coords=[tuple(v.co) for v in o.data.vertices]
  assets.append(dict(name=name,model_id=1100000+index,object_id=63000+index,footprint=footprint,vertices=len(coords),triangles=len(o.data.polygons),bounds=[[min(v[i] for v in coords),max(v[i] for v in coords)] for i in range(3)],purpose=purpose,reference_analysis_ids=reference,collision_requirement=collision,origin='common ground datum; local centered X/Z; rotations cardinal; native128 units/tile',native_shading='explicit material extras; 0 smooth / 1 flat'))
  o.location=((len(assets)-1)%8*9,(len(assets)-1)//8*10,0);o.select_set(False)
 def arch():
  for sign in [-1,1]:
   poly=[(sign*x,z) for x,z in [(.91,0),(1.28,0),(1.25,.13),(1.19,.20),(1.15,1.17),(1.25,1.29),(.91,1.29)]]
   extrusion(poly,.43,'stone')
   extrusion([(sign*x,z) for x,z in [(.88,1.17),(1.28,1.17),(1.29,1.34),(.87,1.34)]],.51,'cap')
  for k in range(9):
   low=k*math.pi/9+.005;high=(k+1)*math.pi/9-.005;r0=.90;r1=1.24+(.045 if k==4 else 0)
   poly=[(r*math.cos(t),1.27+r*math.sin(t)) for r,t in [(r0,low),(r1,low),(r1,high),(r0,high)]]
   extrusion(poly,.45 if k!=4 else .51,['stone','stone_dark','stone','stone_light','cap','stone','stone_light','stone','stone_dark'][k])
  # A localized repair clamp, not a brass stripe around every course.
  box((1.17,-.235,.63),(.15,.05,.36),'iron');box((1.17,-.27,.50),(.06,.04,.06),'iron_light')
 def wall():
  # Continuous shaped body, shallow face colour transitions and one repair joint.
  poly=[(-1,0),(1,0),(1,1.96),(.96,2.08),(-.97,2.10),(-1,1.97)]
  extrusion(poly,.35,'stone')
  box((0,0,.05),(2.02,.48,.30),'stone_dark');box((0,0,2.09),(2.04,.49,.15),'cap')
  for x,z,w,h,mat in [(-.57,.52,.76,.34,'stone_dark'),(.38,1.25,1.05,.42,'stone_light')]:box((x,-.188,z),(w,.03,h),mat)
  for x in [-.70,0,.70]:
   o=extrusion([(x-.10,1.82),(x+.10,1.82),(x+.10,2.02),(x-.10,2.02)],.10,'stone_dark',y=-.22)
  box((.66,-.21,.72),(.025,.035,.65),'mortar',.07)
 def corner():
  poly=[(-1,-1),(1,-1),(1,-.62),(-.62,-.62),(-.62,1),(-1,1)]
  verts=[(x,y,z) for z in [0,2.10] for x,y in poly];n=len(poly);faces=[tuple(range(n-1,-1,-1)),tuple(range(n,2*n))]+[(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
  mesh('bonded corner',verts,faces,'stone')
  box((-.80,-.80,1.12),(.45,.45,2.24),'stone_dark');box((-.80,-.80,2.24),(.56,.56,.17),'cap')
 def fixture():
  box((0,0,.10),(1.65,1.3,.20),'stone_dark')
  lathe((-.2,.10,.18),[(0,.54,0),(.15,.57,0),(1.16,.48,.04),(1.34,.34,.04),(1.39,.28,.04)],'iron',8,smooth=True)
  cylinder((-.2,.10,1.65),.20,.50,'iron',sides=8)
  cylinder((-.2,-.40,.93),.36,.12,'iron_light',axis=(0,1,0),sides=8)
  cylinder((-.2,-.48,.93),.27,.045,'soot',axis=(0,1,0),sides=8)
  beam((-.2,-.515,.93),(-.04,-.515,1.10),.035,'brass')
  cylinder((.43,.10,.64),.17,.80,'iron',axis=(1,0,0),sides=6,smooth=True)
  cylinder((.81,.10,.37),.17,.60,'iron',sides=6,smooth=True)
  cylinder((.81,.10,.73),.23,.07,'iron_light',sides=6)
  beam((.58,.10,.80),(1.02,.10,.80),.07,'wood')
  box((-.16,.61,.60),(.26,.04,.32),'damp')
 asset('arch',10,[3,1],arch,'Worker portal; tapered jambs, restrained stone crown and single localized repair',[1868,2109,7016],'Block jambs only; opening remains traversable; future per-tile support definitions')
 asset('wall',5,[2,1],wall,'Continuous load-bearing masonry, coping and shallow repairs',[6132,6010,6143],'Two tile barrier; visual thickness does not define collision automatically')
 asset('wall_corner',68,[2,2],corner,'Bonded L corner with substantial end pier; complements straight wall',[6001,6132],'Block only L occupied edge cells, not all four cells')
 asset('pressure_pump',17,[2,2],fixture,'Asymmetric pressure vessel with accessible gauge, outlet and shutoff',[9089,9114,9085],'Two tile machine; leave front operating apron clear')
 def figure():
  for x in [-.13,.13]:box((x,0,.39),(.17,.22,.78),'wood')
  extrusion([(-.28,.77),(.28,.77),(.23,1.38),(-.23,1.38)],.31,'cloth')
  lathe((0,0,1.40),[(0,.15,0),(.23,.17,0),(.32,.10,0)],'cap',6)
  for sign in [-1,1]:beam((sign*.24,0,1.3),(sign*.36,0,.85),.12,'wood')
 asset('scale_figure',32,[1,1],figure,'Static 1.72-tile comparison convention; not an NPC or measured original player',[],'No collision; offline ruler only')
 bpy.ops.wm.save_as_mainfile(filepath=str(a.output/'calibrated-kit.blend'))
 (a.output/'kit-manifest.json').write_text(json.dumps(dict(blender=bpy.app.version_string,origin='Entirely original geometry; measured reference analysis only; no proprietary inputs',iteration=a.iteration,palette=COLORS,assets=assets),indent=2));print('CALIBRATED_KIT_SOURCE_PASS',len(assets))
if __name__=='__main__':main()
