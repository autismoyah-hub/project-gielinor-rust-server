"""Read only cache-group identity/header diagnostics, without launching a client."""
from pathlib import Path
import bz2,gzip,hashlib,json


def group(root,archive,index):
    with (root/f'main_file_cache.idx{archive}').open('rb') as f:
        f.seek(index*6);entry=f.read(6)
    if len(entry)!=6: raise ValueError('missing index entry')
    length=int.from_bytes(entry[:3],'big');sector=int.from_bytes(entry[3:],'big')
    if not 0<length<16*1024*1024: raise ValueError('invalid group length')
    data=bytearray();seen=set();part=0
    with (root/'main_file_cache.dat2').open('rb') as f:
        while len(data)<length:
            if sector==0 or sector in seen: raise ValueError('invalid sector chain')
            seen.add(sector);f.seek(sector*520);block=f.read(520)
            if len(block)<8: raise ValueError('truncated sector')
            if int.from_bytes(block[:2],'big')!=index or int.from_bytes(block[2:4],'big')!=part or block[7]!=archive:
                raise ValueError('sector ownership mismatch')
            sector=int.from_bytes(block[4:7],'big');data.extend(block[8:8+min(512,length-len(data))]);part+=1
    compressed=int.from_bytes(data[1:5],'big');kind=data[0]
    if kind==0: payload=bytes(data[5:5+compressed])
    elif kind in (1,2):
        expected=int.from_bytes(data[5:9],'big')
        if expected>16*1024*1024: raise ValueError('oversized unpacked group')
        payload=(bz2.decompress(b'BZh1'+data[9:9+compressed]) if kind==1 else gzip.decompress(data[9:9+compressed]))
        if len(payload)!=expected: raise ValueError('unpacked length mismatch')
    else: raise ValueError('unsupported compression')
    return payload


def file(root,archive,index,file_id):
    data=group(root,255,archive);p=0
    def take(n):
        nonlocal p
        out=data[p:p+n];p+=n
        if len(out)!=n: raise ValueError('truncated reference table')
        return out
    version=take(1)[0]
    if version not in (5,6,7): raise ValueError('unknown reference protocol')
    if version>=6: take(4)
    flags=take(1)[0]
    def count():
        return int.from_bytes(take(4 if version>=7 and data[p]&128 else 2),'big')&0x7fffffff
    n=count();ids=[];value=0
    for _ in range(n): value+=count();ids.append(value)
    if flags&1: take(4*n)
    take(4*n)
    if flags&8: take(4*n)
    if flags&2: take(64*n)
    if flags&4: take(8*n)
    take(4*n)
    counts=[count() for _ in ids];files={}
    for id,total in zip(ids,counts):
        value=0;files[id]=[]
        for _ in range(total): value+=count();files[id].append(value)
    payload=group(root,archive,index);ids=files[index]
    if len(ids)==1:
        if ids[0]!=file_id: raise ValueError('unknown file')
        return payload
    chunks=payload[-1];table=len(payload)-1-4*chunks*len(ids);position=0;offset=table
    result=bytearray()
    for _ in range(chunks):
        size=0
        for id in ids:
            size+=int.from_bytes(payload[offset:offset+4],'big',signed=True);offset+=4
            if size<0 or position+size>table: raise ValueError('invalid group chunk')
            if id==file_id: result.extend(payload[position:position+size])
            position+=size
    if position!=table or file_id not in ids: raise ValueError('invalid group file')
    return bytes(result)


if __name__=='__main__':
    root=Path(__file__).resolve().parent.parent/'work/integration'
    result={}
    for name,path in [('a-kotlin',root/'a-kotlin/.data/cache/js5'),('b-rust',root/'b-rust/cache/js5')]:
        result[name]={}
        for interface in (149,217,162,548):
            data=group(path,3,interface)
            result[name][interface]={'unpacked_group_sha256':hashlib.sha256(data).hexdigest(),'length':len(data),
                'first_bytes':data[:20].hex(),'first_file_if3':data[0]==255,'first_file_kind':data[1] if data[0]==255 else data[0]}
        result[name]['inventory_93_definition_hex']=file(path,2,5,93).hex()
    print(json.dumps(result,indent=2))
