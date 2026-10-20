"""Bounded diagnostic codecs. Never distribute their proprietary inputs/outputs.

Supports classic, untextured models including skin labels and face depth flags.
This is separate from the production GLB opaque-static contract.
"""
import json, struct, sys, re, hashlib
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent))
from convert_glb import smart, require

def encode_model(g):
    vs,fs=g['vertices'],g['faces'];nv,nf=len(vs),len(fs)
    require(0<nv<=65535 and 0<nf<=65535,'count bounds')
    require(not g.get('bone_weights') and not g.get('texture_triangles') and not any(x>=0 for x in g.get('face_textures') or []),'classic untextured only')
    require(all(len(f)==3 and all(0<=i<nv for i in f) for f in fs),'face bounds')
    flags=bytearray();streams=[bytearray() for _ in range(3)];prev=[0]*3
    for vertex in vs:
        require(all(abs(v-round(v))<1e-4 and abs(v)<32768 for v in vertex),'integral native coordinates required')
        v=list(map(round,vertex));flag=0
        for a in range(3):
            delta=v[a]-prev[a]
            if delta:flag|=1<<a;streams[a]+=smart(delta)
        flags.append(flag);prev=v
    indices=bytearray();last=0
    for a,b,c in fs:indices+=smart(a-last)+smart(b-a)+smart(c-b);last=c
    def channel(key,n):
        value=g.get(key)
        require(value is None or len(value)==n,f'{key} count')
        return b'' if value is None else bytes(value)
    rendering=channel('render_types',nf);priorities=channel('priorities',nf)
    skin=channel('vertex_skins',nv);face_skin=channel('face_skins',nf)
    alpha=channel('alpha',nf);depth=channel('face_flags',nf)
    require(all(x in (0,1)for x in rendering),'unsupported render flags')
    require(len(g['colors'])==nf and all(0<=x<=65535 for x in g['colors']),'color bounds')
    lengths=[len(s)for s in streams]+[len(indices),len(skin)]
    require(all(n<=65535 for n in lengths),'section limit')
    # SkinnedLegacy body and 23-byte footer, selected by ff fe marker.
    footer=struct.pack('>HHBBBBBBBHHHHH',nv,nf,0,bool(rendering),255 if priorities else g.get('priority',0),bool(alpha),bool(face_skin),bool(skin),0,*lengths)+b'\xff\xfe'
    return bytes(flags)+bytes([1])*nf+priorities+face_skin+rendering+skin+alpha+indices+struct.pack('>'+str(nf)+'H',*g['colors'])+b''.join(streams)+bytes([bool(depth)])+depth+footer

def encode_animation(data,out,source):
    out.mkdir(parents=True,exist_ok=True);rows=data['frames'];sid=data['custom_skeleton'];types=rows[0]['types'];labels=rows[0]['labels']
    require(len(types)<=255 and len(types)==len(labels),'classic label count')
    skeleton=bytes([len(types)])+bytes(types)+bytes(map(len,labels))+b''.join(bytes(v)for v in labels)
    (out/f'skeleton-{sid}.bin').write_bytes(skeleton)
    for row in rows:
        require(row['types']==types and row['labels']==labels,'mixed skeleton not supported')
        masks=[0]*len(types);values=bytearray()
        for label,v in row['transforms']:
            require(0<=label<len(types) and masks[label]==0,'transform label')
            masks[label]=7
            for x in v:values+=smart(round(x))
        (out/f"frame-{row['index']}.bin").write_bytes(struct.pack('>HB',sid,len(types))+bytes(masks)+values)
    n=len(rows);group=data['custom_frame_group']
    seq=bytes([1])+struct.pack('>H',n)+b''.join(struct.pack('>H',r['length'])for r in rows)+b''.join(struct.pack('>H',r['index'])for r in rows)+struct.pack('>H',group)*n
    # Clock semantics retained from decoded original, not copied binary payloads.
    meta=(source/'sequence.txt').read_text()
    match=re.search(r'interleave: Some\(\s*\[([^]]+)\]',meta,re.S)
    interleave=[int(x)for x in re.findall(r'\d+',match.group(1))if int(x)!=9999999]if match else []
    if data.get('independent'):interleave=[5,7,9,11]if source.name=='attack'else []
    if interleave:seq+=bytes([3,len(interleave),*interleave])
    if 'stretch: true' in meta:seq+=bytes([4])
    priority=re.search(r'\bpriority: (\d+)',meta)
    if priority:seq+=bytes([5,int(priority.group(1))])
    for field,opcode in [('loop_frames',2),('max_loops',8),('precedence',9),('priority_moving',10),('restart',11)]:
        match=re.search(r'\b'+field+r': (-?\d+)',meta);v=int(match.group(1)) if match else -1
        if v>=0:seq+=bytes([opcode])+(struct.pack('>H',v)if opcode==2 else bytes([v]))
    (out/'sequence.bin').write_bytes(seq+b'\0')

if __name__=='__main__':
    source,out=map(Path,sys.argv[1:3]);out.mkdir(parents=True,exist_ok=True)
    for n in ['static','npc']:(out/f'{n}.model').write_bytes(encode_model(json.loads((source/f'{n}.json').read_text())))
    for i,name in enumerate(['idle','walk','attack','hit','death']):
        d=json.loads((source/name/'animation.json').read_text());encode_animation(d,out/name,Path(sys.argv[3])/name)
    independent='independent' in str(source);mid=1100202 if independent else 1100200;nid=60001 if independent else 60000;oid=63201 if independent else 63200
    # Native decoder-compatible standalone diagnostic definitions. Never installed
    # in the ordinary cache or production registry.
    (out/'npc.id').write_text(str(nid))
    (out/'static.id').write_text(str(oid))
    npc=bytes([61,1])+struct.pack('>I',mid+1)+bytes([2])+b'Diagnostic private study\0'+bytes([13])+struct.pack('>H',36181)+bytes([14])+struct.pack('>H',36180)+bytes([100,15,0])
    (out/'npc.definition').write_bytes(npc)
    (out/'static.definition').write_bytes(bytes([6,1])+struct.pack('>I',mid)+bytes([0,14,1,15,1,22,17,19,0,0]))
    registry=dict(scope='standalone native diagnostic loader only; no JS5 installation',npc_id=nid,static_object_id=oid,models={str(mid):'static.model',str(mid+1):'npc.model'},sequences={name:30000+int(re.search(r'\bid: (\d+)',(Path(sys.argv[3])/name/'sequence.txt').read_text()).group(1))for name in ['idle','walk','attack','hit','death']})
    (out/'diagnostic-registry.json').write_text(json.dumps(registry,indent=2))
    print('CALIBRATION_CODEC_COMPLETE')
