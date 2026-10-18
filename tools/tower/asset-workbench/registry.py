"""Bounded immutable native-object packages and collision-safe identity allocation."""
import json,struct,zlib,hashlib
from pathlib import Path
from native_codec import model,require
class IdentityAllocator:
    """Persistent single-writer manifest; names keep identities across refinements."""
    def __init__(self,path,bounds):self.path=Path(path);self.bounds=bounds;self.data=json.loads(self.path.read_text())if self.path.exists()else dict(schema=1,assets={})
    def allocate(self,key):
        if key in self.data['assets']:return self.data['assets'][key]
        n=len(self.data['assets']);row=dict(model_id=1101500+n,object_id=63400+n,npc_id=60020+n,sequence_base=45000+n*8,skeleton_base=64500+n*8,frame_group_base=62300+n*8)
        require(row['object_id']<=64999 and row['npc_id']<=65535 and row['skeleton_base']+7<=65535 and row['frame_group_base']+7<=65535,'custom namespace exhausted; do not wrap IDs')
        for field,source in [('model_id','model_max'),('object_id','object_max'),('npc_id','npc_max'),('sequence_base','sequence_max'),('skeleton_base','skeleton_max'),('frame_group_base','frame_group_max')]:
            if source in self.bounds:require(row[field]>self.bounds[source],'source identifier collision '+field)
        for old in self.data['assets'].values():require(all(row[k]!=old[k]for k in row),'duplicate custom identity')
        self.data['assets'][key]=row;self.path.parent.mkdir(parents=True,exist_ok=True);tmp=self.path.with_suffix('.tmp');tmp.write_text(json.dumps(self.data,indent=2));tmp.replace(self.path);return row
def object_package(out,geometries,factory,model_ids,object_id,original_bounds):
    out=Path(out);out.mkdir(parents=True,exist_ok=True)
    require(len(geometries)==len(model_ids)and len(set(model_ids))==len(model_ids),'registry model cardinality')
    require(all(i>original_bounds['model_max']and i>=1100000 for i in model_ids),'original/custom model collision')
    require(original_bounds['object_max']<object_id<=64999,'original/custom object collision')
    rows=['TOWER_ASSET_PACKAGE 1'];catalog=[]
    for i,g in zip(model_ids,geometries):
        blob=model(g);name=f'model-{i}.model';(out/name).write_bytes(blob);rows.append(f'model {i} {name} {zlib.crc32(blob):08x}');catalog.append(dict(id=i,file=name,sha256=hashlib.sha256(blob).hexdigest()))
    shape=factory.get('shape',10);shaped=shape!=10
    definition=bytes([6 if shaped else 7,len(model_ids)])+b''.join(struct.pack('>I',i)+(bytes([shape])if shaped else b'')for i in model_ids)+b'\2Private diagnostic object\0'
    size=factory.get('size',[1,1]);definition+=bytes([14,size[0],15,size[1],17,19,0])
    if factory.get('rotated'):definition+=bytes([62])
    if factory.get('non_flat_shading'):definition+=bytes([22])
    for k,v in enumerate(factory.get('scale',[128]*3)):definition+=bytes([65+k])+struct.pack('>H',v)
    for k,v in enumerate(factory.get('offset',[0]*3)):definition+=bytes([70+k])+struct.pack('>h',v)
    for key,opcode in [('recolor',40),('retexture',41)]:
        pairs=factory.get(key,[])
        if pairs:definition+=bytes([opcode,len(pairs)])+b''.join(struct.pack('>HH',*p)for p in pairs)
    definition+=b'\0';name=f'object-{object_id}.object';(out/name).write_bytes(definition);rows.append(f'object {object_id} {name} {zlib.crc32(definition):08x}');(out/'registry.txt').write_text('\n'.join(rows)+'\n');(out/'catalog.json').write_text(json.dumps(dict(object_id=object_id,model_ids=model_ids,models=catalog,factory=factory),indent=2))
