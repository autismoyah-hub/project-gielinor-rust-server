"""Walk every authored court through normal movement; optionally capture native frames."""
import argparse, pathlib, json, time
from tower_gateway import Gateway
p = argparse.ArgumentParser()
p.add_argument('--directory', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
p.add_argument('--native-frames', action='store_true')
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
g = Gateway(a.directory, a.output / 'tour-actions.ndjson')
g.wait(lambda s: s.get('ready'), 120)
g.act('close')
g.act('camera 2048 0')
g.interface(35913794)
g.interface(41222162)
points = [('pasture', 10408, 104), ('southern court', 10424, 104), ('poultry yard', 10440, 104), ('waterworks', 10438, 116), ('sanctuary', 10424, 120), ('workshop', 10406, 118), ('grove', 10408, 136), ('foundry entrance', 10423, 132), ('vent gallery', 10438, 132), ('sanctuary return', 10424, 120)]
records = []
for name, x, z in points:
    end = time.monotonic() + 55
    retry = 0
    while True:
        s = g.observe()
        if s['ready'] and s['position'][1:] == [x, z]:
            break
        if time.monotonic() >= end:
            raise TimeoutError(f'Court route failed: {name}')
        if time.monotonic() >= retry:
            g.act(f'walk {x} {z}')
            retry = time.monotonic() + 4
        time.sleep(0.2)
    tick = s['tick']
    g.wait(lambda s: s['tick'] > tick + 1, 6)
    shot = g.act('capture' if a.native_frames else 'observe')
    records.append({'court': name, 'position': shot['position'], 'sequence': shot['sequence']})
    print(name, shot['position'], shot['sequence'], flush=True)
(a.output / 'tour-summary.json').write_text(json.dumps({'accepted': True, 'mode': 'native' if a.native_frames else 'headless', 'courts': records}, indent=2), encoding='utf8')
