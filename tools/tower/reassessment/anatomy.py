"""Reusable static surface authoring. No cache or reference mesh readers.

All dimensions use native units, in Blender X/right, Y/depth, Z/up.
Profile stations, landmarks, surface regions and weapon cross sections are
editable author inputs. They are not recovered vertex arrays.
"""
import sys,math,colorsys
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path[:0]=[str(HERE.parent/'mesh-study'),str(HERE.parent/'high-fidelity')]
from mesh_tools import *
from refine import cage,union
from mathutils import Vector

def native_color(name,code,flat=False):
    return material(name,colorsys.hls_to_rgb(((code>>10)+.2)/64,((code&127)+.2)/127,(((code>>7)&7)+.2)/8),flat)

def longitudinal(name,stations,mat,sides=12):
    """Horizontal organic cage: (depth,center height,half width,half height).

    Explicit ring phase and a common longitudinal axis keep silhouette stations
    stable, unlike a moving Frenet frame around a shoulder or sloping muzzle.
    """
    vs=[(rx*math.cos(a*math.tau/sides),y,z+rz*math.sin(a*math.tau/sides))for y,z,rx,rz in stations for a in range(sides)]
    fs=[tuple(reversed(range(sides))),tuple((len(stations)-1)*sides+j for j in range(sides))]
    fs += [(i*sides+j,i*sides+(j+1)%sides,(i+1)*sides+(j+1)%sides,(i+1)*sides+j)for i in range(len(stations)-1)for j in range(sides)]
    return mesh(name,vs,fs,mat)

def landmark_limb(name,points,radii,mat,voxel=1.8,ratio=.22):
    """Parallel-transport frame avoids the vertical tangent axis-flip defect.

    Ring normals are projected from their predecessor, never independently
    recomputed against world-up. Thus an anatomical joint cannot collapse into
    a bow tie when the limb tangent crosses the up-vector singularity.
    """
    pts=list(map(Vector,points));vs=[];sides=10;normal=Vector((1,0,0))
    for i,(p,(rx,ry))in enumerate(zip(pts,radii)):
        tangent=(pts[min(i+1,len(pts)-1)]-pts[max(i-1,0)]).normalized()
        normal=(normal-tangent*normal.dot(tangent)).normalized()
        if normal.length<.1:normal=tangent.cross(Vector((0,1,0))).normalized()
        other=tangent.cross(normal).normalized()
        vs += [tuple(p+normal*rx*math.cos(j*math.tau/sides)+other*ry*math.sin(j*math.tau/sides))for j in range(sides)]
    fs=[tuple(reversed(range(sides))),tuple((len(pts)-1)*sides+j for j in range(sides))]
    fs += [(i*sides+j,i*sides+(j+1)%sides,(i+1)*sides+(j+1)%sides,(i+1)*sides+j)for i in range(len(pts)-1)for j in range(sides)]
    return mesh(name,vs,fs,mat)

def solid_surface(o,thickness=1):
    mod=o.modifiers.new('Physical thickness and closed boundary','SOLIDIFY');mod.thickness=thickness;mod.offset=0;apply(o,mod);return o

def projected_regions(o,regions):
    """Cut actual faces at projected authored polygon boundaries before paint.

    Region tuples are material, projection axes, polygon, world predicate.
    This avoids a sawtooth color boundary determined by old triangle centroids.
    """
    import bmesh
    bm=bmesh.new();bm.from_mesh(o.data)
    for mat,axes,polygon,predicate in regions:
        for i,a in enumerate(polygon):
            b=polygon[(i+1)%len(polygon)];normal=Vector((0,0,0));normal[axes[0]]=b[1]-a[1];normal[axes[1]]=a[0]-b[0]
            point=Vector((0,0,0));point[axes[0]]=a[0];point[axes[1]]=a[1]
            bmesh.ops.bisect_plane(bm,geom=list(bm.verts)+list(bm.edges)+list(bm.faces),dist=.01,plane_co=o.matrix_world.inverted()@point,plane_no=o.matrix_world.to_3x3().transposed()@normal,clear_inner=False,clear_outer=False)
    bm.to_mesh(o.data);bm.free();o.data.update()
    return surface_regions(o,[(mat,lambda x,y,z,axes=axes,poly=poly,pred=pred:pred(x,y,z)and polygon_contains(((x,y,z)[axes[0]],(x,y,z)[axes[1]]),poly))for mat,axes,poly,pred in regions])

def fitted_patch(name,body,side,outline,mat,offset=.35,axis=0):
    """Lateral facial landmark patch ray-fitted to the actual authored surface.

    The outline lives in depth/height coordinates. Every point intersects the
    head along X so an eye marking cannot become a floating eyeball or clip
    inside a changed skull. No reference geometry participates in the fit.
    """
    vs=[];inverse=body.matrix_world.inverted()
    other=[i for i in range(3)if i!=axis]
    for a,b in outline:
        origin=Vector((0,0,0));origin[axis]=side*200;origin[other[0]]=a;origin[other[1]]=b;direction=Vector((0,0,0));direction[axis]=-side
        hit,pos,normal,_=body.ray_cast(inverse@origin,inverse.to_3x3()@direction)
        if not hit:raise ValueError('Facial landmark misses authored head: '+name)
        world=body.matrix_world@pos;world[axis]+=side*offset;vs.append(tuple(world))
    return solid_surface(mesh(name,vs,[tuple(range(len(vs)))],mat),.25)

def cranial_planes(name,stations,mat):
    """Authored cranium: depth, width, lower jaw line, crown height.

    Eight explicit angular landmarks distinguish crown, orbital plane,
    zygomatic breadth and underside. This is not an elliptical animal template.
    """
    vs=[]
    for y,w,lo,hi in stations:
        h=hi-lo
        vs.extend([(0,y,hi),(.65*w,y,hi-.1*h),(w,y,lo+.55*h),(.8*w,y,lo+.15*h),(0,y,lo),(-.8*w,y,lo+.15*h),(-w,y,lo+.55*h),(-.65*w,y,hi-.1*h)])
    fs=[tuple(reversed(range(8))),tuple((len(stations)-1)*8+j for j in range(8))]
    fs += [(i*8+j,i*8+(j+1)%8,(i+1)*8+(j+1)%8,(i+1)*8+j)for i in range(len(stations)-1)for j in range(8)]
    return mesh(name,vs,fs,mat)

def polygon_contains(point,polygon):
    x,y=point;inside=False;j=len(polygon)-1
    for i,(xi,yi) in enumerate(polygon):
        xj,yj=polygon[j]
        if (yi>y)!=(yj>y) and x < (xj-xi)*(y-yi)/(yj-yi)+xi:inside=not inside
        j=i
    return inside

def surface_regions(o,regions):
    """Assign explicit polygon regions to a continuous mesh, in world space.

    Each (material,predicate) runs in order; no imported UV, texture, face ID or
    proprietary material array is consulted. Mesh precision controls boundary
    resolution. Coarse native faceting remains visible by design.
    """
    for mat,predicate in regions:
        index=len(o.data.materials);o.data.materials.append(mat)
        for p in o.data.polygons:
            q=o.matrix_world@p.center
            if predicate(q.x,q.y,q.z):p.material_index=index
    return o

def shell_profile(name,outline,depth,mat,axis='Y',offset=(0,0,0)):
    """Extruded authored planar silhouette, normals recalculated on creation."""
    n=len(outline)
    def point(x,z,d):
        p=(x,d,z)if axis=='Y'else(d,x,z)if axis=='X'else(x,z,d)
        return tuple(p[i]+offset[i]for i in range(3))
    vs=[point(x,z,d)for d in [-depth/2,depth/2]for x,z in outline]
    fs=[tuple(reversed(range(n))),tuple(n+i for i in range(n))]+[(i,(i+1)%n,(i+1)%n+n,i+n)for i in range(n)]
    return mesh(name,vs,fs,mat)

def curved_blade(name,stations,mat,edge_mat,axis='Y',height=0):
    """Variable diamond-section weapon sweep.

    Stations are (depth,center X,half width,half thickness). Curvature, distal
    taper, spine and cutting edge are independent of hilt orientation.
    """
    vs=[]
    for y,c,w,t in stations:vs.extend([(c-w,y,height),(c,y,height+t),(c+w,y,height),(c,y,height-t)])
    fs=[(3,2,1,0),tuple((len(stations)-1)*4+j for j in range(4))]
    fs += [(i*4+j,i*4+(j+1)%4,(i+1)*4+(j+1)%4,(i+1)*4+j)for i in range(len(stations)-1)for j in range(4)]
    o=mesh(name,vs,fs,mat);o.data.materials.append(edge_mat)
    return o
