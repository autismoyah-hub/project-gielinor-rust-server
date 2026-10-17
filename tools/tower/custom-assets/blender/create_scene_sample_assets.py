"""Scene-calibration additions only. Reads original Tower GLBs, never cache art.

Keeps the unapproved full-floor package immutable. A reproducible rack origin
correction and six genuinely new occupational/detail families form this sample.
"""
import argparse,json,math,sys,shutil,colorsys,hashlib
from pathlib import Path
import bpy,bmesh
from mathutils import Vector,Matrix

def main():
 p=argparse.ArgumentParser();p.add_argument('--input',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--wall-shading',choices=['flat','smooth','selective'],default='flat');a=p.parse_args(sys.argv[sys.argv.index('--')+1:]);a.input=a.input.resolve();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=True)
 manifest=json.loads((a.input/'kit-manifest.json').read_text());assets=[d for d in manifest['assets'] if not d['name'].startswith('ground_')]
 for d in assets:shutil.copy2(a.input/(d['name']+'.glb'),a.output/(d['name']+'.glb'))
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
 colors={'iron':(0,0,28),'iron_edge':(0,0,39),'lamp':(8,1,81),'wood':(6,3,32),'wood_edge':(6,3,39),'cloth':(59,2,30),'cloth_edge':(59,2,39),'stone':(6,0,30),'stone_dark':(6,0,24),'paper':(8,1,63),'pot':(4,3,36)};mats={};parts=[]
 for name,(h,s,l) in colors.items():
  rgb=colorsys.hls_to_rgb((h+.25)/64,(l+.25)/128,(s+.25)/8);rgb=[c/12.92 if c<=.04045 else ((c+.055)/1.055)**2.4 for c in rgb]
  m=bpy.data.materials.new(name);m.diffuse_color=(*rgb,1);m.use_nodes=True;m.use_backface_culling=True;m['native_render_type']=1;m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=(*rgb,1);mats[name]=m
 def finish(o,mat):
  o.data.materials.append(mats[mat]);bpy.context.view_layer.objects.active=o;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
  bm=bmesh.new();bm.from_mesh(o.data);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bmesh.ops.triangulate(bm,faces=list(bm.faces));bm.to_mesh(o.data);bm.free();parts.append(o);o.select_set(False);return o
 def box(pos,size,mat='wood',angle=0):
  bpy.ops.mesh.primitive_cube_add(size=1,location=pos);o=bpy.context.object;o.dimensions=size;o.rotation_euler.z=angle;return finish(o,mat)
 def cylinder(pos,radius,depth,mat='iron',sides=6,axis=(0,0,1),radius2=None):
  bpy.ops.mesh.primitive_cone_add(vertices=sides,radius1=radius,radius2=radius if radius2 is None else radius2,depth=depth,location=pos);o=bpy.context.object;o.rotation_euler=Vector(axis).to_track_quat('Z','Y').to_euler();return finish(o,mat)
 def beam(start,end,width,mat='iron'):
  v=Vector(end)-Vector(start);o=box((Vector(start)+Vector(end))/2,(width,width,v.length),mat);o.rotation_euler=v.to_track_quat('Z','Y').to_euler();return o
 def profile(poly,depth,mat,y=0):
  n=len(poly);v=[(x,yy,z) for yy in [y-depth/2,y+depth/2] for x,z in poly];f=[tuple(range(n-1,-1,-1)),tuple(range(n,2*n))]+[(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)];me=bpy.data.meshes.new('original profile');me.from_pydata(v,[],f);me.update();o=bpy.data.objects.new('original profile',me);bpy.context.collection.objects.link(o);return finish(o,mat)
 def export(name,index,size,build,purpose,refs):
  nonlocal parts
  parts=[];bpy.ops.object.select_all(action='DESELECT');build()
  for o in parts:o.select_set(True)
  bpy.context.view_layer.objects.active=parts[0];bpy.ops.object.join();o=bpy.context.object;o.name=name;bpy.ops.object.transform_apply(location=False,rotation=True,scale=True)
  for v in o.data.vertices:v.co+=o.location
  o.location=(0,0,0);bpy.ops.export_scene.gltf(filepath=str(a.output/(name+'.glb')),export_format='GLB',use_selection=True,export_apply=True,export_yup=True,export_extras=True)
  pts=[tuple(v.co) for v in o.data.vertices];assets.append(dict(name=name,model_id=1100000+index,object_id=63000+index,footprint=size,bounds=[[min(v[i] for v in pts),max(v[i] for v in pts)] for i in range(3)],triangles=len(o.data.polygons),purpose=purpose,reference_analysis_ids=refs,collision_requirement='Offline sample only; wall attached or ground detail as specified; no production collision',origin='Common ground datum; 128 native units per tile; cardinal rotation'))
  o.location=((len(assets)-52)%4*4,5+(len(assets)-52)//4*4,0);o.select_set(False)
 def lantern():
  # Attached on the south face of a wall centered on the same native tile.
  box((0,-.20,1.83),(.14,.14,.17),'iron');beam((0,-.22,1.88),(0,-.52,1.88),.055)
  beam((0,-.51,1.87),(0,-.51,1.66),.035)
  cylinder((0,-.51,1.43),.14,.34,'lamp',sides=6)
  for z in [1.24,1.62]:cylinder((0,-.51,z),.18,.07,'iron_edge',sides=6)
  for x in [-.13,.13]:beam((x,-.58,1.28),(x,-.58,1.58),.035)
 export('wall_lantern',81,[1,1],lantern,'Small wall lantern near working height; pale opaque faces do not emit dynamic light',[3474,206])
 def coats():
  box((0,-.20,1.45),(1.48,.13,.15),'wood')
  for x in [-.40,.38]:
   beam((x,-.23,1.45),(x,-.38,1.45),.045)
   profile([(x-.23,.60),(x+.24,.63),(x+.18,1.20),(x+.33,1.28),(x+.21,1.40),(x-.19,1.39),(x-.32,1.26),(x-.18,1.20)],.055,'cloth',y=-.36)
   box((x,-.40,.91),(.045,.035,.50),'cloth_edge')
 export('wall_coats',82,[2,1],coats,'Two worker coats establish an occupied rest bay without copying cache clothing geometry',[6062])
 def shelf():
  for x in [-.65,.65]:box((x,-.03,.79),(.12,.18,1.58),'wood')
  for z in [.20,.79,1.43]:box((0,-.18,z),(1.52,.46,.11),'wood_edge')
  for x,z in [(-.38,.95),(.29,.95),(.42,.35)]:cylinder((x,-.18,z),.14,.26,'pot',sides=7,radius2=.12)
  for x in [-.37,-.22,-.06]:box((x,-.15,.43),(.09,.27,.36),'paper')
 export('personal_shelf',83,[2,1],shelf,'Pots, ledgers and personal stores grouped on a compact service shelf',[6155,6055])
 def scatter():
  for x,y,w,h,t in [(-.54,.27,.23,.06,.2),(-.18,-.36,.31,.08,.7),(.49,.17,.25,.10,-.2),(.22,.48,.16,.06,.1)]:
   cylinder((x,y,h/2),w/2,h,'stone_dark' if w<.25 else 'stone',sides=5,radius2=w*.34)
 export('ground_scatter',84,[2,2],scatter,'Low edge grit; used only at failure, fuel and tool handling sites',[325,174])
 def rug():
  box((0,0,.023),(2.85,1.72,.035),'cloth')
  for x in [-1.33,1.33]:box((x,0,.045),(.11,1.68,.025),'cloth_edge')
  for y in [-.75,.75]:box((0,y,.045),(2.55,.10,.025),'cloth_edge')
 export('worker_rug',85,[3,2],rug,'Muted, maintained woven rest-bay mat; colored faces only, no copied texture',[])
 def threshold():
  box((0,0,.04),(2.7,.75,.08),'stone_dark');box((-.10,0,.08),(2.28,.71,.045),'stone')
 export('stone_threshold',86,[3,1],threshold,'Low worn threshold locates the entrance and its repair history',[6098])
 # Correct the original tool-rack attachment datum only for this sample package.
 bpy.ops.object.select_all(action='DESELECT');before=set(bpy.data.objects);source=a.input/'supply_rack.glb';bpy.ops.import_scene.gltf(filepath=str(source));imported=[o for o in bpy.data.objects if o not in before and o.type=='MESH']
 for o in imported:
  matrix=o.matrix_world.copy()
  for v in o.data.vertices:v.co=matrix@v.co+Vector((0,-.36,0))
  o.parent=None;o.matrix_world=Matrix.Identity(4);o.select_set(True)
 bpy.context.view_layer.objects.active=imported[0];bpy.ops.object.join();bpy.ops.export_scene.gltf(filepath=str(a.output/'supply_rack.glb'),export_format='GLB',use_selection=True,export_apply=True,export_yup=True,export_extras=True)
 for d in assets:
  if d['name']=='supply_rack':
   d['source_original_glb_sha256']=hashlib.sha256(source.read_bytes()).hexdigest();d['sample_refinement']='Translate all evaluated original vertices -0.36 tile along Blender Y, making the rack meet its wall rather than float in front of it.';d['bounds'][1]=[v-.36 for v in d['bounds'][1]]
 smooth_names=['cave_mass','cave_shoulder']+(['wall','parapet'] if a.wall_shading!='flat' else [])
 for name in smooth_names:
  bpy.ops.object.select_all(action='DESELECT');before=set(bpy.data.objects);bpy.ops.import_scene.gltf(filepath=str(a.input/(name+'.glb')))
  selected=[o for o in bpy.data.objects if o not in before and o.type=='MESH']
  for o in selected:
   o.select_set(True)
   for material in o.data.materials:material['native_render_type']=0
   if name in ['wall','parapet'] and a.wall_shading=='selective':
    for material in o.data.materials:material['native_render_type']=1
    copies={}
    for face in o.data.polygons:
     if abs(face.normal.y)>.98:
      index=face.material_index
      if index not in copies:
       material=o.data.materials[index].copy();material.name+='-planar-native-smooth';material['native_render_type']=0;copies[index]=len(o.data.materials);o.data.materials.append(material)
      face.material_index=copies[index]
  bpy.ops.export_scene.gltf(filepath=str(a.output/(name+'.glb')),export_format='GLB',use_selection=True,export_apply=True,export_yup=True,export_extras=True)
  next(d for d in assets if d['name']==name)['sample_refinement']='Identical original geometry; explicit native smooth render type 0 instead of flat type 1, informed by scene-level cave references. No global light change.'
  if name in ['wall','parapet']:
   entry=next(d for d in assets if d['name']==name);entry['scene_non_flat_shading']=True;entry['sample_refinement']=f'Identical original geometry; {a.wall_shading} native face shading plus definition opcode22. Selective mode keeps closed end/cap faces flat and makes wall planes smooth, preventing their normals being rounded across hard edges. Informed by actual Keldagrim wall6098; no global light change.'
 manifest['assets']=assets;manifest['origin']='Entirely original Tower GLBs and six original scene-calibration additions; no cache input';manifest['sample_only']=True;manifest['parent_manifest_sha256']=hashlib.sha256((a.input/'kit-manifest.json').read_bytes()).hexdigest()
 (a.output/'kit-manifest.json').write_text(json.dumps(manifest,indent=2));bpy.ops.wm.save_as_mainfile(filepath=str(a.output/'scene-sample-additions.blend'));print('SCENE_SAMPLE_ASSETS_PASS',len(assets))
if __name__=='__main__':main()
