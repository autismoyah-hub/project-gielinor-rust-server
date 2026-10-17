"""Verify normal portal exit/return preserves earned character state."""
import argparse, json, pathlib
from tower_gateway import Gateway
p = argparse.ArgumentParser()
p.add_argument('--directory', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
g = Gateway(a.directory, a.output / 'exit-return-actions.ndjson')

def pack(s):
    return [(i['slot'], i['id'], i['count']) for i in s['items']]
before = g.wait(lambda s: s.get('ready') and 'Done: Claim reward' in s['journal'], 90)
g.act('close')
g.act('object 57692 10432 124 1')
away = g.wait(lambda s: s['ready'] and s['position'] == [0, 3224, 3218], 55)
assert pack(before) == pack(away) and before['equipment'] == away['equipment'] and (before['xp'] == away['xp'])
g.act('object 57692 3225 3218 1')
after = g.wait(lambda s: s['ready'] and s['position'] == [0, 10424, 120], 55)
assert pack(before) == pack(after) and before['equipment'] == after['equipment'] and (before['xp'] == after['xp'])
assert before['journal'] == after['journal']
(a.output / 'exit-return-summary.json').write_text(json.dumps({'accepted': True, 'before': before, 'away': away, 'after': after}, indent=2), encoding='utf8')
print('Normal exit and return preserve inventory, equipment, XP and completed run', flush=True)
