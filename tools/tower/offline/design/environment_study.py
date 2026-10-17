"""Compile and render a detached, explicitly authored environmental quality study.

All cache-rendered pixels stay in --output. No production source is loaded or edited.
"""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from PIL import Image


def compile_scene(config, treatment):
    settings = config['treatments'][treatment]
    lines = ['# OFFLINE ENVIRONMENT QUALITY STUDY; NOT A PRODUCTION MAP']
    for x, z, material, rotation in config['chunks']:
        sx, sz = config['materials'][settings.get(material, material)]
        lines.append(f'T {x} {z} 0 {sx} {sz} 0 {rotation}')
    used = set()
    for row in config['objects']:
        x, z, asset, shape, rotation = (row[k] for k in ['x', 'z', 'asset', 'shape', 'rotation'])
        if isinstance(asset, str):
            asset = settings[asset]
        layer = 0 if shape <= 3 else 1 if shape <= 8 else 2 if shape <= 21 else 3
        key = (x, z, layer)
        if key in used:
            raise ValueError(f'Conflicting scene layer at {key}')
        used.add(key)
        lines.append(f'L {x} {z} 0 {asset} {shape} {rotation}')
    for label, (x, z) in config['access_points'].items():
        lines.append(f'P {label} {x} {z}')
    for name, pose in config['views'].items():
        if pose[-1] == 'detail':
            lines.append(f'P view-{name} {pose[0]} {pose[1]}')
    for i, (x, z) in enumerate(config['tour']):
        lines.append(f'P route-{i} {x} {z}')
    for i, (x, z) in enumerate(config['fish_reserves']):
        lines.append(f'F fish-{i} {x} {z}')
    return '\n'.join(lines) + '\n'


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--config', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--treatment', default='dressed')
    p.add_argument('--renderer', type=Path)
    p.add_argument('--cache', type=Path)
    p.add_argument('--views', nargs='+', help='Optional exact view names for treatment comparisons')
    p.add_argument('--assembly', choices=['runtime', 'authored'], default='authored')
    a = p.parse_args()
    config = json.loads(a.config.read_text())
    a.output.mkdir(parents=True, exist_ok=True)
    scene = a.output / 'environment.scene'
    scene.write_text(compile_scene(config, a.treatment))
    manifest = {
        'scope': 'Detached environmental quality study. No production integration or gameplay acceptance.',
        'treatment': a.treatment,
        'object_assembly': a.assembly,
        'config_sha256': hashlib.sha256(a.config.read_bytes()).hexdigest(),
        'scene_sha256': hashlib.sha256(scene.read_bytes()).hexdigest(),
        'views': [],
    }
    if a.renderer:
        if not a.cache:
            p.error('--cache is required with --renderer')
        manifest['renderer_sha256'] = hashlib.sha256(a.renderer.read_bytes()).hexdigest()
        for name, pose in config['views'].items():
            if a.views and name not in a.views:
                continue
            bmp = a.output / f'{name}.bmp'
            result = subprocess.run([
                str(a.renderer.resolve()), str(a.cache.resolve()), str(scene.resolve()),
                str(bmp.resolve()), *map(str, pose), a.assembly
            ], capture_output=True, text=True)
            (a.output / f'{name}.log').write_text(result.stdout + result.stderr)
            result.check_returncode()
            png = bmp.with_suffix('.png')
            Image.open(bmp).save(png, optimize=True)
            manifest['views'].append({
                'name': name, 'camera': pose,
                'raw_sha256': hashlib.sha256(bmp.read_bytes()).hexdigest(),
                'png_sha256': hashlib.sha256(png.read_bytes()).hexdigest(),
            })
            print(a.treatment, name, 'OFFLINE', flush=True)
    (a.output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')


if __name__ == '__main__':
    main()
