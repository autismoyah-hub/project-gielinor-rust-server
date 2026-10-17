"""Validate and register the entirely original modular Floor One kit."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import zlib
from convert_glb import convert,encode,require

def main():
    p=argparse.ArgumentParser();p.add_argument('input',type=Path);p.add_argument('output',type=Path);a=p.parse_args()
    a.output.mkdir(parents=True,exist_ok=True)
    (a.output/'validation').mkdir(exist_ok=True)
    manifest=json.loads((a.input/'kit-manifest.json').read_text())
    rows=['TOWER_ASSET_PACKAGE 1'];catalogue=[];failures=[]
    for index,asset in enumerate(manifest['assets']):
        name=asset['name'];source=a.input/(name+'.glb')
        try:
            vertices,faces,colors,shading=convert(source,with_shading=True);model=encode(vertices,faces,colors,shading)
            require(max(x*x+z*z for x,y,z in vertices)<2900**2,'native model radius limit; partition large geometry before rendering')
            require(model==encode(*convert(source,with_shading=True)),'conversion must be deterministic')
        except Exception as error:
            failures.append(f'{name}: {error}');continue
        model_id=asset.get('model_id',1100000+index);object_id=asset.get('object_id',63000+index)
        require(model_id>=1100000 and 63000<=object_id<=64999,'reserved original-art ID range')
        require(not any(x['model_id']==model_id or x['object_id']==object_id for x in catalogue),'duplicate custom ID')
        expected=''.join('v %d %d %d\n'%v for v in vertices)+''.join('f %d %d %d %d\n'%(*f,c) for f,c in zip(faces,colors))
        (a.output/'validation'/(name+'.txt')).write_text(expected,encoding='ascii')
        (a.output/'validation'/(name+'.shading')).write_text(' '.join(map(str,shading)),encoding='ascii')
        width,length=asset['footprint']
        (a.output/'validation'/(name+'.definition')).write_text(f'{object_id} {model_id} {width} {length} {int(bool(asset.get("scene_non_flat_shading")))}',encoding='ascii')
        definition=bytes([7,1])+struct.pack('>I',model_id)+b'\x02'+name.replace('_',' ').encode('ascii')+b'\0'+bytes([14,width,15,length])+(bytes([22]) if asset.get('scene_non_flat_shading') else b'')+bytes([17,19,0,0])
        for kind,id,extension,content in [('model',model_id,'model',model),('object',object_id,'object',definition)]:
            filename=f'{name.replace("_","-")}.{extension}';(a.output/filename).write_bytes(content)
            rows.append(f'{kind} {id} {filename} {zlib.crc32(content):08x}')
        catalogue.append(dict(asset,model_id=model_id,object_id=object_id,vertices_native=len(vertices),triangles_native=len(faces),palette=sorted(set(colors)),glb_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),model_sha256=hashlib.sha256(model).hexdigest(),model_bytes=len(model)))
    require(not failures,'\n'.join(failures))
    (a.output/'registry.txt').write_text('\n'.join(rows)+'\n',encoding='ascii')
    (a.output/'catalogue.json').write_text(json.dumps(dict(origin=manifest['origin'],blender=manifest['blender'],palette=manifest['palette'],assets=catalogue),indent=2)+'\n')
    print('ORIGINAL_KIT_CONVERSION_PASS',len(catalogue),'assets',sum(a['triangles_native'] for a in catalogue),'unique model triangles')

if __name__=='__main__':main()
