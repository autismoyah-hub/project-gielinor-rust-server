"""Validate actual sanctuary returns, open cross bays and normal service access."""
import argparse
import csv
import json
import pathlib
import time
from tower_gateway import Gateway

p = argparse.ArgumentParser()
p.add_argument('--directory', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
p.add_argument('--label', default='rebuilt')
p.add_argument('--baseline', action='store_true')
p.add_argument('--native-frames', action='store_true')
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
g = Gateway(a.directory, a.output / f'{a.label}-sanctuary-actions.ndjson')
before = g.wait(lambda s: s['ready'] and s['tick'] > 0 and 'Cinderworks' in s['journal'], 120)
g.act('inspect')
rows = list(csv.DictReader((a.directory / 'scene-objects.csv').open(encoding='utf8', newline='')))
actual = []
for x in (10420, 10428):
    for z in (118, 121, 122):
        matches = [r for r in rows if r['plane'] == '0' and r['x'] == str(x) and r['z'] == str(z) and r['shape'] == '0' and r['rotation'] == '0' and r['rejected'] == 'false']
        if a.baseline:
            assert not any(int(r['id']) == 6143 for r in matches)
        else:
            assert len(matches) == 1 and int(matches[0]['id']) == 6143
            actual.append(matches[0])
    for z in (119, 120):
        assert not any(r['plane'] == '0' and r['x'] == str(x) and r['z'] == str(z) and r['shape'] == '0' and r['id'] == '6143' for r in rows), 'Main cross bay closed'

def walk(x, z):
    end = time.monotonic() + 65
    retry = 0
    while True:
        s = g.observe()
        if s['ready'] and s['position'][1:] == [x, z]:
            return s
        if time.monotonic() > end:
            raise TimeoutError(f'Normal sanctuary route blocked: {x},{z}')
        if time.monotonic() > retry:
            g.act(f'walk {x} {z}')
            retry = time.monotonic() + 4
        time.sleep(.2)

visits = []
for x, z in [(10418, 120), (10430, 120), (10424, 120), (10422, 124)]:
    visits.append(walk(x, z)['position'])
g.act('close')
g.act('object 18491 10421 124 2')
bank = g.wait(lambda s: s['ready'] and bool(s['modals']), 35)
frames = []
if a.native_frames:
    tick = bank['tick']
    g.wait(lambda s: s['tick'] > tick + 2 and bool(s['modals']), 10)
    s = g.act('capture')
    f = a.directory / f"frame-{s['sequence']}.bmp"
    end = time.monotonic() + 10
    while not f.exists():
        if time.monotonic() > end:
            raise TimeoutError('No genuine bank UI capture')
        time.sleep(.1)
    frames.append(str(f.resolve()))
g.act('close')
g.wait(lambda s: not s['modals'], 20)
visits.append(walk(10427, 116)['position'])
g.act('object 409 10428 116 1')
tick = g.observe()['tick']
g.wait(lambda s: s['ready'] and s['tick'] > tick + 2, 15)
visits.append(walk(10424, 120)['position'])
after = g.observe()
def pack(s):
    return sorted((i['slot'], i['id'], i['count']) for i in s['items'] if i['count'] > 0)
assert pack(before) == pack(after) and before['equipment'] == after['equipment'] and before['xp'] == after['xp'] and before['journal'] == after['journal']
(a.output / f'{a.label}-sanctuary-summary.json').write_text(json.dumps({'accepted': True, 'baseline': a.baseline, 'actual': actual, 'crossing_gaps_open': True, 'normal_routes': visits, 'bank_modal_ids': bank['modals'], 'bank_action': 'Normal displayed Bank operation2 on18491', 'altar_action': 'Normal Pray operation1 on409 from its actual adjacent western tile', 'exact_items_equipment_all25XP_journal_preserved': True, 'native_bank_frames': frames}, indent=2), encoding='utf8')
print('Real Bank/Pray and sanctuary crossings preserve exact earned state', flush=True)
