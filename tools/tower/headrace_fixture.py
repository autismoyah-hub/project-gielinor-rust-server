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
for z in [119, 120, 121]:
    matches = [r for r in rows if r['plane'] == '0' and r['x'] == '10445' and (r['z'] == str(z)) and (r['shape'] == '0') and (r['rotation'] == '0') and (r['rejected'] == 'false')]
    assert len(matches) == 1 and int(matches[0]['id']) == 6143
    actual.append(matches[0])
assert not any((r['id'] == '28649' and r['x'] == '10445' and (r['z'] == '119') for r in rows)), 'Rejected pipe present'
frames = []
if a.native_frames:
    g.interface(35913794)
    g.interface(41222162)
    g.act('walk 10444 122')
    s = g.wait(lambda s: s['ready'] and s['position'][1:] == [10444, 122], 65)
    tick = s['tick']
    g.wait(lambda s: s['tick'] > tick + 3, 10)
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
(a.output / f'{a.label}-headrace-summary.json').write_text(json.dumps({'accepted': True, 'mode': 'native' if a.native_frames else 'headless', 'actual': actual, 'rejected_pipe_absent': True, 'frame_paths': frames, 'exact_state_preserved': True, 'state': after}, indent=2), encoding='utf8')
print('Actual three low bank edges shape0rotation0 reconstructed; rejected pipe absent', flush=True)
