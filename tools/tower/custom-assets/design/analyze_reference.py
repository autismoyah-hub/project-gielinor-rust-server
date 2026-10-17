"""Publish safe measurements only; never copy source geometry or reference pixels."""
import argparse,json,hashlib,collections
from pathlib import Path

def main():
 p=argparse.ArgumentParser();p.add_argument('reference',type=Path);p.add_argument('symbols',type=Path);p.add_argument('output',type=Path);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
 symbols={int(row.split('\t')[0]):row.rstrip().split('\t')[1] for row in a.symbols.read_text().splitlines() if '\t' in row}
 entries=[]
 for path in sorted(a.reference.glob('object-*.json')):
  d=json.loads(path.read_text());g=d['geometry'];lo=[min(v[i] for v in g['vertices']) for i in range(3)];hi=[max(v[i] for v in g['vertices']) for i in range(3)];edges=collections.Counter(tuple(sorted((u,v))) for f in g['faces'] for u,v in zip(f,f[1:]+f[:1]))
  entries.append(dict(object_id=d['object_id'],name=d['name'],symbol=symbols.get(d['object_id']),model_ids=d['model_ids'],shape=d['shape'],rotation=0,definition_tiles=d['definition_tiles'],vertices=len(g['vertices']),faces=len(g['faces']),bounds_native=[lo,hi],visible_extent_tiles=[round((hi[i]-lo[i])/128,5) for i in range(3)],colors_unique=len(set(g['colors'])),packed_hsl_lightness_range=[min(c&127 for c in g['colors']),max(c&127 for c in g['colors'])],texture_ids=sorted(set(x for x in (g['face_textures'] or []) if x>=0)),render_type_counts=dict(collections.Counter(g['render_types'] or [0]*len(g['faces']))),ambient=d['ambient'],contrast=d['contrast'],non_flat_shading=d['non_flat_shading'],scale=d['scale'],offset=d['offset'],edge_incidence_counts=dict(collections.Counter(edges.values())),local_interchange_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),inspection='3 native Rust views; 4 Blender topology views; preserved local only'))
 manifest=dict(revision='239.1',source='Authorized staged paired cache/js5; read-only dat2/idx',coordinates='128 units per tile; X east, Y negative up, Z north',source_format='Legacy u16 disk revision validated against low bits of reference version plus full group CRC; expanded only in memory; normal client strict validation unchanged',distribution='Measurements and IDs only. No proprietary mesh, texture or reference image included.',sample_limits='Thirty targeted industrial/architectural objects. Symbol names suggest asset families, not verified world placements. Static object compositions at rotation zero, not animation/lifecycle acceptance. 1.72-tile custom scale figure is a comparison convention, not measured universal player height.',assets=entries)
 (a.output/'REFERENCE_MANIFEST.json').write_text(json.dumps(manifest,indent=2)+'\n')
 print('SAFE_REFERENCE_MANIFEST_PASS',len(entries),'objects; no geometry arrays or images')
if __name__=='__main__':main()
