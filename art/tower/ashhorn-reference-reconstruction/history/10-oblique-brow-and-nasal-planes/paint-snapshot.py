"""Visibility-aware original-reference palette guidance on real3D surfaces.

No texture planes or renderer changes. Source illumination is compressed into
restrained material variation rather than copied as a full baked light map.
"""
import bpy,math,colorsys,json
from pathlib import Path
from mathutils import Vector
from mathutils.bvhtree import BVHTree
import numpy as np
import surface as s
def apply(C,reference_dir):
    objects=[o for o in bpy.context.scene.objects if o.type=='MESH' and not o.get('construction_only',False)]
    # Actual mesh triangles receive colors; no hidden billboard participates.
    for o in objects:
        mod=o.modifiers.new('Editable face-color tessellation','TRIANGULATE');s.apply(o,mod)
    vertices=[];faces=[]
    for o in objects:
        offset=len(vertices);vertices.extend(tuple(o.matrix_world@v.co)for v in o.data.vertices);faces.extend(tuple(offset+i for i in p.vertices)for p in o.data.polygons)
    tree=BVHTree.FromPolygons(vertices,faces,all_triangles=True)
    refs={}
    for view,cam in C['cameras'].items():
        im=bpy.data.images.load(str((Path(reference_dir)/(view+'-panel.png')).resolve()),check_existing=True);w,h=im.size;pixels=np.array(im.pixels[:],dtype=float).reshape(h,w,4)[::-1,:,:]
        refs[view]=(cam,np.asarray(cam['eye_author']),np.asarray(cam['basis_author']),pixels)
    palette={};statistics={'painted_faces':0,'retained_semantic_faces':0,'no_visible_reference':0,'source_views':{},'method':'Visibility-aware material-class/chromaticity guidance, source tonal variation compressed; no direct texture/billboard, no renderer lighting cancellation'}
    for o in objects:
        fixed=o.name.startswith(('eye_','orbital_','nostril_','long_','broken_','jagged_','folded_','mouth_')) or '_claw_'in o.name
        if fixed:statistics['retained_semantic_faces']+=len(o.data.polygons);continue
        armor='shingle' in o.name;old=list(o.data.materials)
        for poly in o.data.polygons:
            if old[poly.material_index].name.startswith(('dark','edge')):
                statistics['retained_semantic_faces']+=1;continue
            center=o.matrix_world@poly.center;normal=(o.matrix_world.to_3x3()@poly.normal).normalized();best=None
            for view,(cam,eye,basis,pixels)in refs.items():
                d=eye-np.array(center);length=np.linalg.norm(d);direction=d/length;facing=float(np.dot(normal,direction))
                if facing<.10:continue
                rel=(np.asarray(center)-eye)@basis.T;u=cam['viewport'][0]/2+cam['offset_pixels'][0]+rel[0]*cam['focal']/rel[2];v=cam['viewport'][1]/2+cam['offset_pixels'][1]-rel[1]*cam['focal']/rel[2]
                x,y=round(u),round(v)
                if not(2<=x<pixels.shape[1]-2 and 2<=y<pixels.shape[0]-2):continue
                sample=pixels[y-1:y+2,x-1:x+2,:];valid=sample[:,:,3]>.8
                if valid.sum()<5:continue
                hit,_,_,distance=tree.ray_cast(Vector(eye),Vector(-direction),length+3)
                if hit is not None and distance<length-1.7:continue
                rgb=np.median(sample[:,:,:3][valid],axis=0);r,g,b=rgb
                if armor and not(r>g*1.12 and g>b*1.08):continue
                weight=facing**2*(1.1 if view in ['front','left'] else .8)
                if best is None or weight>best[0]:best=(weight,view,rgb)
            if best is None:
                statistics['no_visible_reference']+=1
                if C.get('paint_semantic_constraints',False):best=(0,'semantic fallback',np.array([.33,.33,.31]))
                else:continue
            _,view,rgb=best;r,g,b=rgb;light=(float(max(rgb))+float(min(rgb)))/2
            semantic=C.get('paint_semantic_constraints',False)
            if semantic:
                x,y,z=center;warm=False
                if o.name=='angular_mandible':warm=True
                elif o.name=='continuous_anatomy':
                    warm=(abs(x)<30 and ((y<-30 and 55<z<121)or(-30<=y<99 and 55<z<82)))
                    for side,key2 in [(1,'pos'),(-1,'neg')]:
                        for kind,a,b,zlo,zhi in [('fore','elbow','wrist',25,68),('hind','hip','knee',43,95)]:
                            pa=C['landmarks'][a+'_'+key2];pb=C['landmarks'][b+'_'+key2]
                            if zlo<z<zhi:
                                t=max(0,min(1,(z-pb[2])/max(.1,pa[2]-pb[2])));cx=pb[0]+(pa[0]-pb[0])*t;cy=pb[1]+(pa[1]-pb[1])*t
                                if abs(y-cy)<16 and 0<side*x<abs(cx)-3:warm=True
                if armor:
                    rusty=(r-g)>.13 and r>.43;h,ss,l=(.078,.56,.37)if rusty else(.073,.46,.26);factor=max(.88,min(1.12,.95+.10*(light/.32)))
                elif warm:h,ss,l=.095,.27,.34;factor=max(.92,min(1.08,.97+.06*(light/.36)))
                else:
                    h,ss,l=.09,.065,.28
                    # Reject color contamination from nearby claws, horns,
                    # armor or ventral skin at an approximate projection edge.
                    factor=max(.92,min(1.08,.97+.06*(light/.33)))if r<g*1.14 else 1.0
            elif armor:
                rusty=(r-g)>.13 and r>.43
                h,ss,l=(.078,.56,.43)if rusty else(.073,.46,.31);factor=max(.74,min(1.24,.85+.35*(light/.32)))
            elif r>g*1.14 and g>b*1.13:
                h,ss,l=.095,.30,.43;factor=max(.74,min(1.23,.82+.38*(light/.36)))
            else:
                h,ss,l=.09,.075,.37;factor=max(.76,min(1.24,.80+.40*(light/.33)))
            value=min(.70,l*factor);code=(int(h*64)<<10)|(int(ss*8)<<7)|int(value*127)//3*3
            flat=bool(old[poly.material_index].get('native_flat',False));key=(code,flat)
            if key not in palette:palette[key]=s.native_color('Original reference albedo '+str(code)+(' flat'if flat else''),code,flat)
            mat=palette[key]
            index=o.data.materials.find(mat.name)
            if index<0:index=len(o.data.materials);o.data.materials.append(mat)
            poly.material_index=index;statistics['painted_faces']+=1;statistics['source_views'][view]=statistics['source_views'].get(view,0)+1
    statistics['native_palette_entries']=len(palette);return statistics
