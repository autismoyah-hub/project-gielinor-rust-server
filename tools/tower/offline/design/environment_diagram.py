"""Original geometric diagrams only: does not open any cache or rendered media."""
import argparse
import json
import re
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--config', type=Path, required=True)
    p.add_argument('--audit', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    c = json.loads(a.config.read_text())
    audit = a.audit.read_text()
    if 'issues=0' not in audit:
        raise ValueError('Diagram requires a passing static audit')
    defs = {int(i): (int(w), int(h)) for i, w, h in re.findall(r'^ASSET (\d+) ".*?" (\d+)x(\d+)', audit, re.M)}
    scale, ox, oy = 25, 70, 145
    im = Image.new('RGB', (1120, 1320), '#15212a')
    d = ImageDraw.Draw(im)
    font = ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf', 21)
    small = ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf', 17)
    title = ImageFont.truetype('C:/Windows/Fonts/segoeuib.ttf', 32)
    d.text((45, 24), 'The Hearth and Headgate', font=title, fill='#f5d49a')
    d.text((45, 74), 'Connected environmental study · original diagram · north up', font=font, fill='#cad8df')
    d.text((45, 108), 'Cache-rendered close views remain in the local review gallery.', font=small, fill='#a9bdc9')

    def xy(x, z):
        return ox + (x - 40) * scale, oy + (72 - z) * scale

    def rect(x, z, w=1, h=1):
        left, bottom = xy(x, z)
        return left, bottom - h * scale, left + w * scale, bottom

    colors = {'paving': '#627681', 'court-floor': '#627681', 'work-floor': '#4a4244', 'water': '#326f88'}
    for x, z, material, rotation in c['chunks']:
        d.rectangle(rect(x * 8, z * 8, 8, 8), fill=colors[material], outline='#233741')
    settings = c['treatments']['dressed']
    for row in c['objects']:
        x, z, shape, rotation = (row[k] for k in ['x', 'z', 'shape', 'rotation'])
        if shape > 3:
            continue
        left, top, right, bottom = rect(x, z)
        edges = [[(left, top), (left, bottom)], [(left, top), (right, top)], [(right, top), (right, bottom)], [(left, bottom), (right, bottom)]]
        for side in [rotation, (rotation + 1) % 4] if shape == 2 else [rotation]:
            d.line(edges[side], fill='#ddd1b8', width=5)
    for row in c['objects']:
        if row['shape'] < 9:
            continue
        asset = row['asset']
        asset = settings[asset] if isinstance(asset, str) else asset
        w, h = defs.get(asset, (1, 1))
        if row['rotation'] % 2:
            w, h = h, w
        box = rect(row['x'], row['z'], w, h)
        color = '#c89c60' if 'bench' in row['label'] or 'seat' in row['label'] else '#e4b663' if 'hearth' in row['label'] or 'fire' in row['label'] else '#8db47c' if 'fern' in row['label'] else '#b5b5a5'
        if asset == 7017:
            d.ellipse(box, fill=color, outline='#263b45', width=2)
        else:
            d.rectangle(box, fill=color, outline='#263b45', width=2)
    path = [(int(x), int(z)) for x, z in re.findall(r'^PATH \d+ (\d+) (\d+)', audit, re.M)]
    d.line([(xy(x + .5, z + .5)) for x, z in path], fill='#f8cb7c', width=3)
    labels = [('Entrance', (48, 36)), ('Sanctuary', (48, 51)), ('Workshop', (61, 47)), ('Waterworks', (66, 60)), ('Quiet return', (52, 59)), ('Foundry threshold', (64, 67))]
    for i, (name, (x, z)) in enumerate(labels, 1):
        px, py = xy(x + .5, z + .5)
        d.ellipse((px - 15, py - 15, px + 15, py + 15), fill='#f7d495', outline='#1a2d38', width=2)
        d.text((px - 6, py - 15), str(i), font=font, fill='#192c36')
    for i, (name, _) in enumerate(labels, 1):
        d.text((55 + ((i - 1) % 3) * 350, 1190 + ((i - 1) // 3) * 38), f'{i}. {name}', font=font, fill='#e0e9ed')
    d.text((55, 1280), 'Gold: audited tour · geometric furniture footprints · no cache pixels or model silhouettes', font=small, fill='#a9bdc9')
    a.output.mkdir(parents=True, exist_ok=True)
    im.save(a.output / 'sequence-plan.png', optimize=True)
    detail = Image.new('RGB', (1680, 720), '#15212a')
    draw = ImageDraw.Draw(detail)
    # Crop only this newly drawn geometric diagram, never cache-derived imagery.
    for index, (label, bounds) in enumerate([
        ('Sanctuary: refuge and service alcoves', (40, 40, 56, 57)),
        ('Workshop: shaping, heat and finishing', (55, 41, 71, 55)),
        ('Waterworks: wheel, repair bay and return', (56, 54, 72, 71)),
    ]):
        x0, z0, x1, z1 = bounds
        l, b = xy(x0, z0)
        r, t = xy(x1, z1)
        crop = im.crop((l, t, r, b))
        crop.thumbnail((520, 570))
        detail.paste(crop, (index * 560 + 20, 100))
        draw.text((index * 560 + 20, 28), label, font=small, fill='#f5d49a')
    draw.text((25, 680), 'Original footprint details. Artistic asset comparison requires the authentic local renders.', font=font, fill='#bfced7')
    detail.save(a.output / 'sequence-details.png', optimize=True)


if __name__ == '__main__':
    main()
