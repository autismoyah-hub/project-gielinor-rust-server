"""Original-only immutable overlay and per-object native collision identities."""
from pathlib import Path
import json,struct,zlib,hashlib,shutil
T=Path(__file__).resolve().parent;S=T.parents[2];R=S.parent/'cinderworks-environment-private';P=S/'art/tower/cinderworks-environment'
NAMES=['wall-high','wall-low','pier','arch','furnace-bank','water-main','pipe-elbow','rail-span','ore-cart','slate-headwall','railing','sluice','stock-crate','tool-rack','moss-fern','ash-rubble','paving-bond','reward-surround','pump','plate-mill','workstation','water-return','arch-foot','chimney-backdrop','arrival-inlay','hearth-ring']
SIZES={'furnace-bank':[3,2],'water-main':[1,2],'pipe-elbow':[2,2],'rail-span':[2,2],'ore-cart':[2,2],'slate-headwall':[3,1],'railing':[2,1],'sluice':[2,1],'paving-bond':[2,2],'pump':[2,2],'plate-mill':[2,2],'workstation':[2,1],'water-return':[4,1],'hearth-ring':[2,2]}
DECOR={'arch','rail-span','moss-fern','ash-rubble','paving-bond','reward-surround','arrival-inlay','hearth-ring'}
def generate():
    out=R/'package-v1';out.mkdir(exist_ok=True);rows=['TOWER_ASSET_PACKAGE 1'];catalog={}
    for i,name in enumerate(NAMES):
        mid=1220000+i;oid=64700+i;source=next(R/v/name for v in['kit-v4','kit-v3','kit-v2','kit-v1']if(R/v/name/'asset.model').exists());blob=(source/'asset.model').read_bytes();mf=f'model-{mid}.model';(out/mf).write_bytes(blob);rows.append(f'model {mid} {mf} {zlib.crc32(blob):08x}')
        shaped=name.startswith('wall-');ground=name in['rail-span','moss-fern','ash-rubble','paving-bond','arrival-inlay','hearth-ring'];shapes=[0,2]if shaped else[10,22]if ground else[10]
        definition=bytes([6,len(shapes)])+b''.join(struct.pack('>IB',mid,s)for s in shapes)if shaped or ground else bytes([7,1])+struct.pack('>I',mid)
        definition+=bytes([2])+('Cinderworks '+name.replace('-',' ')).encode()+b'\0';w,h=SIZES.get(name,[1,1]);definition+=bytes([14,w,15,h,19,0,22])
        if name in DECOR:definition+=bytes([17])
        if name=='reward-surround':definition+=bytes([72])+struct.pack('>h',128)
        if name=='water-return':definition+=bytes([71])+struct.pack('>h',-240)
        definition+=b'\0';fn=f'object-{oid}.object';(out/fn).write_bytes(definition);rows.append(f'object {oid} {fn} {zlib.crc32(definition):08x}')
        g=json.loads((source/'geometry.json').read_text());catalog[name]={'object_id':oid,'model_id':mid,'size':[w,h],'blocking':name not in DECOR,'source':str(source.relative_to(R)),'vertices':len(g['vertices']),'triangles':len(g['faces']),'model_sha256':hashlib.sha256(blob).hexdigest(),'shapes':shapes,'rotation_convention':'quarter turns, existing native placement transform','animation_anchor':None,'gameplay_hook':None,'interaction':'Decorative original scenery; existing functional objects remain separate'}
    (out/'registry.txt').write_text('\n'.join(rows)+'\n');(out/'catalog.json').write_text(json.dumps(catalog,indent=2)+'\n');print('ORIGINAL_PACKAGE',len(catalog),'models and matching collision definitions');return catalog
if __name__=='__main__':generate()
