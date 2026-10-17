"""Accept real vent defeat and sanctuary rescue without changing server state."""
import argparse, json, pathlib, time
from tower_gateway import Gateway
p = argparse.ArgumentParser()
p.add_argument('--directory', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
g = Gateway(a.directory, a.output / 'recovery-actions.ndjson')
s = g.wait(lambda s: s.get('ready') and 'Done: Altar:' in s['journal'], 90)
before = s
assert any((i['id'] == 1205 for i in s['equipment']))
g.act('close')
g.act('walk 10439 136')
s = g.wait(lambda s: s['position'][1:] == [10439, 136], 55)
imp = next((n for n in s['npcs'] if n['id'] == 16300))
g.act(f"npc {imp['index']} 2")
s = g.wait(lambda s: any((v['id'] == 86 for v in s['graphics'])), 40)
g.act(f"walk {s['position'][1]} {s['position'][2]}")
dead = g.wait(lambda s: s['hp'] == 0, 100)
after = g.wait(lambda s: s['hp'] > 0 and s['position'][1:] == [10424, 120], 30)
assert after['items'] == before['items'], 'Rescue changed inventory'
assert after['equipment'] == before['equipment'], 'Rescue changed equipment'
assert all((label in after['journal'] for label in ['Done: Workshop:', 'Done: Kitchen:', 'Done: Altar:']))
assert 'Next: Silence Ash Imp' in after['journal'], 'Failure granted an unlock'
assert after['hp'] == after['base'][3]
(a.output / 'recovery-summary.json').write_text(json.dumps({'accepted': True, 'before': before, 'defeat': dead, 'after': after}, indent=2), encoding='utf8')
print('Actual lethal vents and sanctuary recovery confirmed; items, equipment and services preserved', flush=True)
