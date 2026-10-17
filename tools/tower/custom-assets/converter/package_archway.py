"""Create append-only experimental definitions and checksum registry."""
import argparse
from pathlib import Path
import struct
import zlib

p=argparse.ArgumentParser()
p.add_argument('directory',type=Path)
a=p.parse_args()
root=a.directory
# Opcode7 carries u32 model IDs. Four tiles wide; visual mesh never blocks
# the passage. Shape22 support locations separately clip the two columns.
arch=bytes([7,1])+struct.pack('>I',1000000)+b'\x02Cinderworks original arch\0'+bytes([14,4,15,1,17,19,1,0])
support=b'\x02Arch column footing\0'+bytes([27,18,19,0,0])
(root/'archway.object').write_bytes(arch)
(root/'column.object').write_bytes(support)
rows=['TOWER_ASSET_PACKAGE 1']
for kind,id,name in [('model',1000000,'archway.model'),('object',65000,'archway.object'),('object',65001,'column.object')]:
    rows.append(f'{kind} {id} {name} {zlib.crc32((root/name).read_bytes()):08x}')
(root/'registry.txt').write_text('\n'.join(rows)+'\n',encoding='ascii')
print('\n'.join(rows))
