"""Merge explicit native type-0 mappings on owned components without losing IDs.

Texture pixels are not read or copied. A texture ID requires the authorized local
cache at render time. B/reference interchange uses its existing unchanged path.
"""
def merge(target,part,vertex_offset,prior_faces):
    textures=part.get('face_textures');coords=part.get('texture_coords');count=len(part['faces']);mapping_offset=len(target['texture_types'])
    if textures is not None and target['face_textures']is None:
        target['face_textures']=[-1]*prior_faces;target['texture_coords']=[-1]*prior_faces
    if target['face_textures']is not None:
        target['face_textures']+=textures if textures is not None else[-1]*count
        target['texture_coords']+=[c+mapping_offset if c>=0 else-1 for c in(coords or[-1]*count)]
    types=part.get('texture_types')or[];triangles=part.get('texture_triangles')or[]
    if len(types)!=len(triangles)or any(t!=0 for t in types):raise ValueError('Only explicit simple native texture maps are authorable')
    target['texture_types']+=types;target['texture_triangles'] += [[v+vertex_offset for v in tri]for tri in triangles]

def apply(part,mat,spec):
    texture=mat.get('native_texture_id')
    if texture is None:return part
    if not isinstance(texture,int)or not 0<=texture<=65534:raise ValueError('Native texture identity range')
    triangles=spec.get('native_texture_triangles',[])
    if spec.get('native_uv_projection'):
        projection=spec['native_uv_projection'];u,v=projection['axes'];normal=next(i for i in range(3)if i not in [u,v]);lo=[min(p[i]for p in part['vertices'])for i in range(3)];hi=[max(p[i]for p in part['vertices'])for i in range(3)]
        if u==v or hi[u]==lo[u]or hi[v]==lo[v]:raise ValueError('Owned planar texture projection axes')
        origin=[(lo[i]+hi[i])*.5 for i in range(3)];origin[u]=lo[u];origin[v]=lo[v];along_u=list(origin);along_v=list(origin);along_u[u]=hi[u];along_v[v]=hi[v];offset=len(part['vertices']);part['vertices'] += [list(map(round,p))for p in [origin,along_u,along_v]]
        if part.get('vertex_skins')is not None:part['vertex_skins'] += [part['vertex_skins'][0]]*3
        triangles=[[offset,offset+1,offset+2]]
    if len(triangles)>255 or any(len(t)!=3 or any(not 0<=i<len(part['vertices'])for i in t)for t in triangles):raise ValueError('Owned mapping triangle bounds')
    coords=spec.get('native_texture_coords',[0 if spec.get('native_uv_projection')else-1]*len(part['faces']))
    if len(coords)!=len(part['faces'])or any(not -1<=i<len(triangles)for i in coords):raise ValueError('Owned face mapping index')
    part.update(face_textures=[texture]*len(part['faces']),texture_coords=coords,texture_types=[0]*len(triangles),texture_triangles=triangles,colors=[mat.get('texture_lightness',127)]*len(part['faces']));return part
