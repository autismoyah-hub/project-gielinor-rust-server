import argparse, csv, json, pathlib, time
from tower_gateway import Gateway
p = argparse.ArgumentParser()
p.add_argument('--directory', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
p.add_argument('--label', default='rebuilt')
p.add_argument('--native-frames', action='store_true')
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
g = Gateway(a.directory, a.output / f'{a.label}-headrace-actions.ndjson')
before = g.wait(lambda s: s['ready'] and 'Done: Claim reward' in s['journal'], 120)
g.act('inspect')
time.sleep(0.3)
rows = list(csv.DictReader((a.directory / 'scene-objects.csv').open(encoding='utf8', newline='')))
actual = []
# A real continuous parapet protects the unsupported source-height seam.
# Existing3tile fishing reach is unchanged; every player approach stays flat.
for z in range(112, 128):
    matches = [r for r in rows if r['plane'] == '0' and r['x'] == '10438' and r['z'] == str(z) and r['shape'] == '0' and r['rotation'] == '2' and r['rejected'] == 'false']
    assert len(matches) == 1 and int(matches[0]['id']) == 6143
    actual.append(matches[0])
for z, angle in [(112, 3), (127, 1)]:
    assert any(r['id'] == '6143' and r['x'] == '10439' and r['z'] == str(z) and r['shape'] == '0' and r['rotation'] == str(angle) and r['rejected'] == 'false' for r in rows)
for z in [121, 125]:
    assert any(r['id'] == '7017' and r['x'] == '10438' and r['z'] == str(z) and r['shape'] == '10' and r['rejected'] == 'false' for r in rows)
wheel = [r for r in rows if r['plane'] == '0' and r['x'] == '10440' and r['z'] == '122' and r['shape'] == '10' and r['rejected'] == 'false']
assert len(wheel) == 1 and int(wheel[0]['id']) in (18511, 18512)
assert not any(r['id'] == '28649' and r['x'] == '10445' and r['z'] == '119' for r in rows), 'Rejected pipe present'
visits = []
for x, z in [(10438, 111), (10438, 129), (10438, 116)]:
    g.act(f'walk {x} {z}')
    s = g.wait(lambda s: s['ready'] and s['position'][1:] == [x, z], 65)
    visits.append(s['position'])
spots = sorted((n['x'], n['z']) for n in s['npcs'] if n['id'] == 3317)
assert spots == [(10440, 114), (10440, 116), (10440, 118), (10440, 120)], spots
# Ordinary walking must not enter the visually sloping bulkhead strip.
g.act('walk 10439 116')
tick = g.observe()['tick']
s = g.wait(lambda s: s['ready'] and s['tick'] > tick + 4, 15)
assert s['position'][1:] == [10438, 116], 'Player crossed quay safety edge'
frames = []
if a.native_frames:
    g.interface(35913794)
    g.interface(41222162)
    g.act('walk 10437 122')
    s = g.wait(lambda s: s['ready'] and s['position'][1:] == [10437, 122], 65)
    tick, cycle = s['tick'], s['cycle']
    g.wait(lambda s: s['ready'] and s['tick'] > tick + 8 and s['cycle'] > cycle + 150 and s['position'][1:] == [10437, 122], 25)
    for yaw in [0, 4096, 8192, 12288]:
        g.act(f'camera 2048 {yaw}')
        time.sleep(0.3)
        s = g.act('capture')
        f = a.directory / f"frame-{s['sequence']}.bmp"
        end = time.monotonic() + 10
        while not f.exists():
            if time.monotonic() > end:
                raise TimeoutError('No genuine headrace native frame')
            time.sleep(0.1)
        frames.append({'yaw': yaw, 'frame': str(f.resolve()), 'sequence': s['sequence']})
    g.act('walk 10424 120')
    g.wait(lambda s: s['ready'] and s['position'][1:] == [10424, 120], 65)
after = g.observe()
pack = lambda s: sorted(((i['slot'], i['id'], i['count']) for i in s['items'] if i['count'] > 0))
assert pack(before) == pack(after) and before['equipment'] == after['equipment'] and (before['xp'] == after['xp']) and (before['journal'] == after['journal'])
(a.output / f'{a.label}-headrace-summary.json').write_text(json.dumps({'accepted': True, 'mode': 'native' if a.native_frames else 'headless', 'actual': actual, 'fish_spots': spots, 'wheel': wheel, 'flat_fishing_approaches_and_guarded_bulkhead': True, 'north_south_crossings': visits, 'rejected_pipe_absent': True, 'frame_paths': frames, 'exact_state_preserved': True, 'state': after}, indent=2), encoding='utf8')
print('Actual guarded quay, flat fishing approaches, through-routes, shoreline wheel and supports reconstructed', flush=True)
