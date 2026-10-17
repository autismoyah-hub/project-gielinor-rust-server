"""Check real processional hall edges, open cross bays and unchanged earned state."""
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
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
g = Gateway(a.directory, a.output / f'{a.label}-hall-actions.ndjson')
before = g.wait(lambda s: s['ready'] and s['tick'] > 0 and 'Done: Claim reward' in s['journal'], 120)
g.act('inspect')
rows = list(csv.DictReader((a.directory / 'scene-objects.csv').open(encoding='utf8', newline='')))
actual = []
for x in (10421, 10427):
    for z in (100, 101, 102, 106, 107, 108):
        matches = [r for r in rows if r['plane'] == '0' and r['x'] == str(x) and r['z'] == str(z) and r['shape'] == '0' and r['rotation'] == '0' and r['rejected'] == 'false']
        assert len(matches) == 1 and int(matches[0]['id']) == 6143
        actual.append(matches[0])
    for z in (103, 104, 105, 109, 110, 111):
        assert not any(r['plane'] == '0' and r['x'] == str(x) and r['z'] == str(z) and r['shape'] == '0' and r['id'] == '6143' for r in rows), 'Cross bay unexpectedly closed'
points = [(10420, 104), (10428, 104), (10420, 104), (10420, 110), (10428, 110), (10420, 110), (10424, 100), (10424, 110), (10424, 120)]
visits = []
for x, z in points:
    end = time.monotonic() + 65
    retry = 0
    while True:
        s = g.observe()
        if s['ready'] and s['position'][1:] == [x, z]:
            visits.append(s['position'])
            break
        if time.monotonic() > end:
            raise TimeoutError(f'Normal hall bay route blocked: {x},{z}')
        if time.monotonic() > retry:
            g.act(f'walk {x} {z}')
            retry = time.monotonic() + 4
        time.sleep(.2)
after = g.observe()
def pack(s):
    return sorted((i['slot'], i['id'], i['count']) for i in s['items'] if i['count'] > 0)
assert pack(before) == pack(after) and before['equipment'] == after['equipment'] and before['xp'] == after['xp'] and before['journal'] == after['journal']
(a.output / f'{a.label}-hall-summary.json').write_text(json.dumps({'accepted': True, 'actual': actual, 'open_cross_bays': True, 'normal_bay_routes': visits, 'exact_earned_state_preserved': True}, indent=2), encoding='utf8')
print('Twelve actual supported hall rails, open cross bays and normal routes confirmed', flush=True)
