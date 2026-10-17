"""Strict untextured glTF 2 GLB to the client's legacy model codec.

Only a deliberately bounded, static, opaque triangle subset is accepted. No
third-party asset or Blender module is needed. Units: glTF metres = tiles.
"""
import argparse
import colorsys
import hashlib
import json
import math
from pathlib import Path
import struct
import zlib


def require(condition, message):
    if not condition:
        raise ValueError(message)


def smart(value):
    require(-16384 <= value < 16384, 'signed-smart delta overflow')
    return bytes([value + 64]) if -64 <= value < 64 else struct.pack('>H', value + 49152)


def encode(vertices, faces, colors, render_types=None):
    require(0 < len(vertices) <= 65535 and 0 < len(faces) <= 65535, 'model count limit')
    render_types = [1]*len(faces) if render_types is None else render_types
    require(len(render_types)==len(faces) and all(t in (0,1) for t in render_types), 'unsupported native shading type')
    flags = bytearray()
    streams = [bytearray(), bytearray(), bytearray()]
    previous = (0, 0, 0)
    for vertex in vertices:
        flag = 0
        for axis in range(3):
            delta = vertex[axis] - previous[axis]
            if delta:
                flag |= 1 << axis
                streams[axis] += smart(delta)
        flags.append(flag)
        previous = vertex
    indices = bytearray()
    last = 0
    for a, b, c in faces:
        indices += smart(a-last) + smart(b-a) + smart(c-b)
        last = c
    require(all(len(s) <= 65535 for s in [*streams, indices]), 'model section limit')
    footer = struct.pack('>HHBBBBBBHHHH', len(vertices), len(faces), 0, 1, 0, 0, 0, 0,
                         *(len(s) for s in streams), len(indices))
    return bytes(flags) + bytes([1])*len(faces) + bytes(render_types) + indices + b''.join(
        struct.pack('>H', c) for c in colors) + b''.join(streams) + footer


def palette(rgba):
    require(len(rgba) == 4 and all(math.isfinite(x) and 0 <= x <= 1 for x in rgba)
            and rgba[3] == 1, 'only opaque finite material factors supported')
    rgb = [12.92*x if x <= .0031308 else 1.055*x**(1/2.4)-.055 for x in rgba[:3]]
    h, l, s = colorsys.rgb_to_hls(*rgb)
    return (min(63, int(h*64)) << 10) | (min(7, int(s*8)) << 7) | min(127, int(l*128))


def convert(path, with_shading=False):
    blob = path.read_bytes()
    magic, version, length = struct.unpack_from('<4sII', blob)
    require(magic == b'glTF' and version == 2 and length == len(blob), 'invalid GLB header')
    offset, chunks = 12, {}
    while offset < length:
        size, kind = struct.unpack_from('<II', blob, offset)
        require(size % 4 == 0 and offset+8+size <= length and kind not in chunks, 'GLB chunk bounds')
        chunks[kind] = blob[offset+8:offset+8+size]
        offset += 8+size
    doc = json.loads(chunks[0x4E4F534A])
    binary = chunks[0x004E4942]
    require(doc['asset']['version'] == '2.0' and not doc.get('extensionsRequired'), 'unsupported glTF extensions')
    require(not any(doc.get(k) for k in ['animations', 'skins', 'textures', 'images']), 'unsupported animated/textured asset')
    require(len(doc['buffers']) == 1 and 'uri' not in doc['buffers'][0], 'only embedded GLB buffer supported')

    def accessor(index, expected):
        a = doc['accessors'][index]
        require(a['type'] == expected and not a.get('sparse') and not a.get('normalized'), 'unsupported accessor')
        component = a['componentType']
        formats = {5126: ('f', 4), 5125: ('I', 4), 5123: ('H', 2), 5121: ('B', 1)}
        require(component in formats, 'unsupported component')
        require(component == 5126 if expected == 'VEC3' else component != 5126, 'accessor component mismatch')
        fmt, size = formats[component]
        n = 3 if expected == 'VEC3' else 1
        view = doc['bufferViews'][a['bufferView']]
        require(view['buffer'] == 0 and not view.get('extensions'), 'unsupported buffer view')
        stride = view.get('byteStride', n*size)
        require(stride >= n*size and stride % size == 0, 'invalid stride')
        count = a['count']
        start = a.get('byteOffset', 0)
        require(0 < count <= 200000 and start >= 0 and start+(count-1)*stride+n*size <= view['byteLength'], 'accessor bounds')
        start += view.get('byteOffset', 0)
        require(start+(count-1)*stride+n*size <= len(binary), 'buffer bounds')
        return [struct.unpack_from('<'+fmt*n, binary, start+i*stride) for i in range(count)]

    def matmul(a, b):
        return [[sum(a[i][k]*b[k][j] for k in range(4)) for j in range(4)] for i in range(4)]

    identity = [[float(i == j) for j in range(4)] for i in range(4)]
    vertices, faces, colors, visited = [], [], [], set()
    render_types=[]
    lookup = {}

    def visit(index, parent):
        require(index not in visited, 'cyclic or instanced node unsupported')
        visited.add(index)
        node = doc['nodes'][index]
        require('skin' not in node and not node.get('weights'), 'skinned/morphed node')
        if 'matrix' in node:
            require(not any(k in node for k in ['translation', 'rotation', 'scale']), 'mixed transform')
            values = node['matrix']
            require(len(values) == 16, 'matrix size')
            local = [[values[j*4+i] for j in range(4)] for i in range(4)]
        else:
            x,y,z,w = node.get('rotation', [0,0,0,1])
            require(abs(x*x+y*y+z*z+w*w-1) < 1e-4, 'nonunit rotation')
            local = [[1-2*(y*y+z*z),2*(x*y-z*w),2*(x*z+y*w),0],
                     [2*(x*y+z*w),1-2*(x*x+z*z),2*(y*z-x*w),0],
                     [2*(x*z-y*w),2*(y*z+x*w),1-2*(x*x+y*y),0],[0,0,0,1]]
            scale = node.get('scale', [1,1,1])
            for i in range(3):
                for j in range(3): local[i][j] *= scale[j]
                local[i][3] = node.get('translation', [0,0,0])[i]
        transform = matmul(parent, local)
        require(all(math.isfinite(v) for row in transform for v in row) and transform[3] == [0,0,0,1], 'invalid affine transform')
        det = sum(transform[0][i]*(transform[1][(i+1)%3]*transform[2][(i+2)%3]-transform[1][(i+2)%3]*transform[2][(i+1)%3]) for i in range(3))
        require(abs(det) > 1e-8, 'singular transform')
        if 'mesh' in node:
            for p in doc['meshes'][node['mesh']]['primitives']:
                require(p.get('mode',4) == 4 and not p.get('targets') and not p.get('extensions'), 'only static triangles supported')
                raw = accessor(p['attributes']['POSITION'], 'VEC3')
                mapped = []
                for v in raw:
                    q = [sum(transform[i][j]*v[j] for j in range(3))+transform[i][3] for i in range(3)]
                    require(all(math.isfinite(c) and abs(c) <= 64 for c in q), 'coordinate bounds')
                    native = tuple(round(c*128) for c in [q[0], -q[1], -q[2]])
                    if native not in lookup:
                        lookup[native] = len(vertices)
                        vertices.append(native)
                    mapped.append(lookup[native])
                indices = [i[0] for i in accessor(p['indices'], 'SCALAR')]
                require(len(indices)%3 == 0 and all(i < len(raw) for i in indices), 'triangle index bounds')
                material = doc['materials'][p['material']]
                require(material.get('alphaMode','OPAQUE') == 'OPAQUE' and not material.get('doubleSided') and not material.get('extensions'), 'unsupported material state')
                pbr = material.get('pbrMetallicRoughness',{})
                require(not any('Texture' in k for k in pbr) and not any('Texture' in k for k in material), 'unsupported material texture')
                color = palette(pbr.get('baseColorFactor',[1,1,1,1]))
                shading=material.get('extras',{}).get('native_render_type',1)
                require(type(shading) is int and shading in (0,1),'unsupported explicit native material shading')
                for k in range(0,len(indices),3):
                    f = tuple(mapped[i] for i in indices[k:k+3])
                    if det < 0: f = f[0],f[2],f[1]
                    a,b,c = [vertices[i] for i in f]
                    u,v = [b[i]-a[i] for i in range(3)],[c[i]-a[i] for i in range(3)]
                    require(any(u[i]*v[(i+1)%3]-u[(i+1)%3]*v[i] for i in range(3)), 'triangle collapsed after quantization')
                    faces.append(f); colors.append(color);render_types.append(shading)
        for child in node.get('children',[]): visit(child, transform)

    for root in doc['scenes'][doc.get('scene',0)]['nodes']: visit(root, identity)
    edges = {}
    for f in faces:
        for a,b in zip(f, f[1:]+f[:1]):
            edges.setdefault(tuple(sorted((a,b))), []).append((a,b))
    require(all(len(e) == 2 and e[0] == e[1][::-1] for e in edges.values()), 'mesh must be closed with consistently oriented faces')
    return (vertices,faces,colors,render_types) if with_shading else (vertices,faces,colors)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('input', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    vertices,faces,colors = convert(args.input)
    encoded = encode(vertices,faces,colors)
    require(encoded == encode(*convert(args.input)), 'nondeterministic conversion')
    args.output.mkdir(parents=True,exist_ok=True)
    (args.output/'archway.model').write_bytes(encoded)
    expected = dict(vertices=vertices,faces=faces,colors=colors)
    (args.output/'expected-geometry.json').write_text(json.dumps(expected),encoding='utf-8')
    # Line-oriented expected data is consumed independently by the native Rust decoder.
    (args.output/'expected-geometry.txt').write_text(''.join('v %d %d %d\n'%v for v in vertices)+''.join('f %d %d %d %d\n'%(*f,c) for f,c in zip(faces,colors)),encoding='ascii')
    report = dict(source_sha256=hashlib.sha256(args.input.read_bytes()).hexdigest(),
                  model_sha256=hashlib.sha256(encoded).hexdigest(),model_crc32=f'{zlib.crc32(encoded):08x}',
                  vertices=len(vertices),triangles=len(faces),palette=sorted(set(colors)),
                  bounds_native=[[min(v[i] for v in vertices),max(v[i] for v in vertices)] for i in range(3)],
                  encoding='Legacy; flat per-face HSL; signed smart deltas',quantization='nearest integer; 128 native units per tile',
                  closed_consistent_winding=True,deterministic=True)
    (args.output/'conversion.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))


if __name__ == '__main__': main()
