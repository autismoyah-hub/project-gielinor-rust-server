"""Explicit extended-model and classic-frame authoring codecs.

No cache access. Simple native texture mappings, face/vertex labels and depth
flags are supported; unavailable complex texture or weighted-bone data is rejected.
"""
import struct,math,json
def require(ok,message):
    if not ok:raise ValueError(message)
def smart(value):
    require(isinstance(value,int)and -16384<=value<16384,'signed-smart overflow')
    return bytes([value+64])if -64<=value<64 else struct.pack('>H',value+49152)
def model(g):
    vs,fs=g['vertices'],g['faces'];nv,nf=len(vs),len(fs);require(0<nv<=65535 and 0<nf<=65535,'model count limit');require(not g.get('bone_weights'),'weighted bone encoding not supported')
    ts=g.get('texture_types')or [];tri=g.get('texture_triangles')or [];require(len(ts)==len(tri)<=255 and all(t==0 for t in ts),'complex texture mappings cannot be reconstructed from omitted decoder data')
    flags=bytearray();xyz=[bytearray()for _ in range(3)];previous=[0,0,0]
    for row in vs:
        require(len(row)==3 and all(math.isfinite(x)and abs(x-round(x))<1e-4 and abs(x)<32768 for x in row),'fractional/non-finite/out-of-range native coordinate; export raw model plus definition transforms instead of silently quantizing')
        row=list(map(round,row));flag=0
        for a in range(3):
            d=row[a]-previous[a]
            if d:flag|=1<<a;xyz[a]+=smart(d)
        flags.append(flag);previous=row
    indices=bytearray();last=0
    for f in fs:
        require(len(f)==3 and all(isinstance(x,int)and 0<=x<nv for x in f),'triangle bounds');a,b,c=f;indices+=smart(a-last)+smart(b-a)+smart(c-b);last=c
    def channel(key,n):
        v=g.get(key);require(v is None or len(v)==n,key+' count');return b''if v is None else bytes(v)
    render=channel('render_types',nf);prio=channel('priorities',nf);alpha=channel('alpha',nf);vskin=channel('vertex_skins',nv);fskin=channel('face_skins',nf);depth=channel('face_flags',nf)
    require(all(x<12 for x in prio),'native priority range')
    tex=g.get('face_textures');coord=g.get('texture_coords');texture_data=b'';coord_data=bytearray()
    if tex is not None:
        require(len(tex)==nf,'texture face count');require(coord is None or len(coord)==nf,'texture coordinate face count');texture_data=b''.join(struct.pack('>H',x+1)for x in tex)
        if ts:
            for i,t in enumerate(tex):
                if t>=0:
                    v=-1 if coord is None else coord[i];require(-1<=v<len(ts),'texture mapping index');coord_data.append(v+1)
    require(len(g['colors'])==nf and all(0<=x<=65535 for x in g['colors']),'HSL face count/range')
    require(all(len(t)==3 and all(0<=i<nv for i in t)for t in tri),'texture triangle bounds')
    lengths=[len(v)for v in xyz]+[len(indices),len(coord_data),len(vskin)];require(all(x<=65535 for x in lengths),'model section overflow')
    metadata=g.get('metadata');meta=b'\0'if metadata is None else b'\1'+struct.pack('>HHHI',*metadata[0],metadata[1])
    body=bytes(ts)+flags+render+bytes([1])*nf+prio+fskin+vskin+alpha+indices+texture_data+coord_data+struct.pack('>'+str(nf)+'H',*g['colors'])+b''.join(xyz)+b''.join(struct.pack('>HHH',*t)for t in tri)+meta+bytes([bool(depth)])+depth
    # SkinnedExtended has one additional bone flag and a bounded vertex section.
    footer=struct.pack('>HHBBBBBBBBHHHHHH',nv,nf,len(ts),bool(render),255 if prio else g.get('priority',0),bool(alpha),bool(fskin),tex is not None,bool(vskin),0,*lengths)+b'\xff\xfd'
    return body+footer
def classic(data,out):
    """Compile a classic action from evaluated Blender channels and explicit policy."""
    from pathlib import Path
    out=Path(out);out.mkdir(parents=True,exist_ok=True);rows=data['frames'];require(bool(rows),'empty action');types=rows[0]['types'];labels=rows[0]['labels'];sid=data['skeleton_id'];group=data['frame_group'];require(0<=sid<=65535 and 0<=group<=65535 and len(types)<=255 and len(labels)==len(types),'classic header bounds')
    require(all(t in (0,1,2,3,5)for t in types),'unsupported transform kind')
    (out/f'skeleton-{sid}.bin').write_bytes(bytes([len(types)])+bytes(types)+bytes(map(len,labels))+b''.join(bytes(v)for v in labels))
    for i,row in enumerate(rows):
        require(row['types']==types and row['labels']==labels,'mixed action skeleton')
        masks=[0]*len(types);values=bytearray();last=-1
        for label,v in row['transforms']:
            require(last<label<len(types)and len(v)==3,'ordered unique transform labels');masks[label]=7;last=label
            for x in v:require(abs(x-round(x))<1e-4,'native action sample must be integral');values+=smart(round(x))
        (out/f'frame-{i}.bin').write_bytes(struct.pack('>HB',sid,len(types))+bytes(masks)+values)
    n=len(rows);require(n<=65535 and all(1<=r['length']<=65535 for r in rows),'action duration bounds')
    seq=b'\1'+struct.pack('>H',n)+b''.join(struct.pack('>H',r['length'])for r in rows)+b''.join(struct.pack('>H',i)for i in range(n))+struct.pack('>H',group)*n
    policy=data.get('policy',{})
    if policy.get('loop_frames',-1)>=0:seq+=b'\2'+struct.pack('>H',policy['loop_frames'])
    if policy.get('interleave'):v=policy['interleave'];require(v==sorted(set(v))and len(v)<=255,'interleave order');seq+=bytes([3,len(v),*v])
    if policy.get('stretch'):seq+=b'\4'
    for key,opcode,default in [('priority',5,5),('max_loops',8,99),('precedence',9,0),('priority_moving',10,0),('restart',11,2)]:seq+=bytes([opcode,policy.get(key,default)])
    (out/'sequence.bin').write_bytes(seq+b'\0')
