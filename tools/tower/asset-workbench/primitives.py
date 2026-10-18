"""Reusable owned-geometry primitives in native coordinates, 128 units per tile."""
import math
def loft(sections,sides=6,outline=None):
    if outline:sides=len(outline)
    if not 3<=sides<=32 or len(sections)<2:raise ValueError('loft requires >=2 sections and 3..32 sides')
    vertices=[];faces=[]
    for row in sections:
        x,y,z,rx,rz=row
        if rx<=0 or rz<=0:raise ValueError('loft radii must be positive; use a cap instead of duplicate apex vertices')
        profile=outline or [[math.cos(i*math.tau/sides),math.sin(i*math.tau/sides)]for i in range(sides)]
        vertices.extend([[round(x+rx*u),round(y),round(z+rz*v)]for u,v in profile])
    for k in range(len(sections)-1):
        for i in range(sides):
            a=k*sides+i;b=k*sides+(i+1)%sides;c=b+sides;d=a+sides;faces.extend([[a,b,c],[a,c,d]])
    for off,reverse in [(0,True),((len(sections)-1)*sides,False)]:
        for i in range(1,sides-1):faces.append([off,off+i+1,off+i]if reverse else [off,off+i,off+i+1])
    return vertices,faces
def box(center,size):
    x,y,z=center;sx,sy,sz=[v/2 for v in size];v=[[round(x+a*sx),round(y+b*sy),round(z+c*sz)]for a,b,c in [(-1,-1,-1),(1,-1,-1),(1,-1,1),(-1,-1,1),(-1,1,-1),(1,1,-1),(1,1,1),(-1,1,1)]];return v,[[0,2,1],[0,3,2],[4,5,6],[4,6,7],[0,1,5],[0,5,4],[1,2,6],[1,6,5],[2,3,7],[2,7,6],[3,0,4],[3,4,7]]
def beam(start,end,radii,sides=6):
    axis=[end[i]-start[i]for i in range(3)];length=math.sqrt(sum(x*x for x in axis))
    if length==0:raise ValueError('zero-length beam')
    axis=[x/length for x in axis];up=[0,0,1]if abs(axis[2])<.9 else [1,0,0]
    cross=lambda a,b:[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
    u=cross(axis,up);norm=math.sqrt(sum(x*x for x in u));u=[x/norm for x in u];w=cross(axis,u);vertices=[]
    for center,radius in [(start,radii[0]),(end,radii[1])]:
        vertices.extend([[round(center[a]+radius*(u[a]*math.cos(i*math.tau/sides)+w[a]*math.sin(i*math.tau/sides)))for a in range(3)]for i in range(sides)])
    _,faces=loft([[0,0,0,1,1],[0,1,0,1,1]],sides);return vertices,faces
def profile(points,depth):
    """Simple polygon extrusion. Blender supplies controlled cap triangulation."""
    from mathutils import Vector
    from mathutils.geometry import tessellate_polygon
    v=[[x,y,z]for z in [-depth/2,depth/2]for x,y in points];n=len(points);faces=[]
    for i in range(n):j=(i+1)%n;faces.extend([[i,j,j+n],[i,j+n,i+n]])
    for off,reverse in [(0,True),(n,False)]:
        poly=[Vector(x)for x in v[off:off+n]]
        for tri in tessellate_polygon([poly]):
            ix=[off+(a if isinstance(a,int)else poly.index(a))for a in tri];faces.append(list(reversed(ix))if reverse else ix)
    return [[round(a)for a in row]for row in v],faces
def socket(vertices,spec):
    origin=spec.get('origin',[0,0,0]);basis=spec.get('basis',[[1,0,0],[0,1,0],[0,0,1]])
    for a in range(3):
        for b in range(3):
            if abs(sum(basis[a][i]*basis[b][i]for i in range(3))-(1 if a==b else 0))>1e-5:raise ValueError('socket basis must be orthonormal')
    det=basis[0][0]*(basis[1][1]*basis[2][2]-basis[1][2]*basis[2][1])-basis[0][1]*(basis[1][0]*basis[2][2]-basis[1][2]*basis[2][0])+basis[0][2]*(basis[1][0]*basis[2][1]-basis[1][1]*basis[2][0])
    if det<.999:raise ValueError('socket reflection would reverse winding')
    return [[round(origin[a]+sum(row[i]*basis[i][a]for i in range(3)))for a in range(3)]for row in vertices]
def make(spec):
    kind=spec['type']
    if kind in ['section_mesh','shell','gear']:
        from advanced_geometry import sections,shell,gear
        if kind=='section_mesh':v,f=sections(spec['sections'],spec.get('profiles'),spec.get('sides',8),spec.get('hem'))
        elif kind=='shell':v,f=shell(spec['sections'],spec.get('thickness',3),spec.get('sides',8),spec.get('start_angle',0),spec.get('end_angle',math.tau))
        else:v,f=gear(spec['center'],spec['radius'],spec['root_radius'],spec['teeth'],spec['depth'],spec.get('bore',0))
    elif kind in ['sweep','stave','voussoir','stone_block']:
        from shape_families import sweep,stave,voussoir,chamfer_block
        if kind=='sweep':v,f=sweep(spec['sections'],spec.get('sides',6),spec.get('normal',[0,0,1]))
        elif kind=='stave':v,f=stave(spec['sections'],spec['start_angle'],spec['end_angle'],spec.get('thickness',4))
        elif kind=='voussoir':v,f=voussoir(spec['center'],spec['inner'],spec['outer'],spec['start_angle'],spec['end_angle'],spec['depth'])
        else:v,f=chamfer_block(spec['center'],spec['size'],spec.get('bevel',3),spec.get('wear',0),spec.get('seed',0))
    elif kind=='loft':v,f=loft(spec['sections'],spec.get('sides',6),spec.get('outline'))
    elif kind=='annulus':
        n=spec.get('sides',12);rx,ry=spec['radii'];width=spec['rim_width'];depth=spec['depth'];cx,cy,cz=spec['center'];v=[];f=[]
        if min(rx,ry)<=width or depth<=0:raise ValueError('annulus dimensions')
        for z,r in [(-depth/2,0),(-depth/2,width),(depth/2,0),(depth/2,width)]:v += [[round(cx+(rx-r)*math.cos(i*math.tau/n)),round(cy+(ry-r)*math.sin(i*math.tau/n)),round(cz+z)]for i in range(n)]
        for i in range(n):
            j=(i+1)%n
            for a,b,c,d in [(i,j,n+j,n+i),(2*n+i,3*n+i,3*n+j,2*n+j),(i,2*n+i,2*n+j,j),(n+i,n+j,3*n+j,3*n+i)]:f.extend([[a,b,c],[a,c,d]])
    elif kind=='box':v,f=box(spec['center'],spec['size'])
    elif kind=='beam':v,f=beam(spec['start'],spec['end'],spec['radii'],spec.get('sides',6))
    elif kind=='profile':v,f=profile(spec['profile'],spec['depth'])
    else:raise ValueError('unknown primitive '+kind)
    if spec.get('deformation_cage'):
        from advanced_geometry import cage
        v=cage(v,spec['deformation_cage'])
    v=socket(socket(v,spec.get('socket',{})),spec.get('parent_socket',{}))
    if spec.get('asset_transform'):
        transform=spec['asset_transform'];scale=transform['scale'];offset=transform['offset']
        if min(scale)<=0:raise ValueError('Owned asset scale must preserve winding')
        v=[[round(row[i]*scale[i]+offset[i])for i in range(3)]for row in v]
    return v,f
