"""Owned-asset lattice welding. No changes to native codec or renderer.

Weld only within one authored component, preserving every classic group and
material boundary. This removes coincident integer vertices after quantization
so the native light interpolator sees a continuous surface at those seams.
"""
import json,sys,copy
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path.insert(0,str(HERE.parent/'asset-workbench'))
from native_codec import model

def compact(g,report):
    result=copy.deepcopy(g);result['vertices']=[];result['vertex_skins']=[];mapping={};start=0;new_components=[]
    for component in report['components']:
        lookup={};first=len(result['vertices'])
        for i in range(start,start+component['vertices']):
            key=tuple(g['vertices'][i])
            if key not in lookup:
                lookup[key]=len(result['vertices']);result['vertices'].append(g['vertices'][i]);result['vertex_skins'].append(g['vertex_skins'][i])
            mapping[i]=lookup[key]
        start+=component['vertices'];c=dict(component);c['vertices']=len(result['vertices'])-first;new_components.append(c)
    assert start==len(g['vertices']);result['faces']=[[mapping[i]for i in f]for f in g['faces']]
    assert all(len(set(f))==3 for f in result['faces'])
    report=dict(report);report['components']=new_components;report['delivery_welded_vertices']=len(g['vertices'])-len(result['vertices']);report['vertices']=len(result['vertices']);report['weld_policy']='Coincident quantized vertices within each authored component only; no cross-joint or cross-component weld.'
    return result,report

if __name__=='__main__':
    source,dest=map(Path,sys.argv[1:]);dest.mkdir(parents=True,exist_ok=True);g,report=compact(json.loads((source/'geometry.json').read_text()),json.loads((source/'conversion.json').read_text()));(dest/'geometry.json').write_text(json.dumps(g));(dest/'conversion.json').write_text(json.dumps(report,indent=2));(dest/'asset.model').write_bytes(model(g));print('WELDED',report['delivery_welded_vertices'])
