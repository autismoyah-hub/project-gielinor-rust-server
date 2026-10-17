"""Claim once through the real chest, optionally proving full-pack atomicity."""
import argparse, pathlib, json, time
from tower_gateway import Gateway
p = argparse.ArgumentParser()
p.add_argument('--directory', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
p.add_argument('--native-frames', action='store_true')
p.add_argument('--coins', type=int, default=500)
p.add_argument('--full-pack', action='store_true')
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
g = Gateway(a.directory, a.output / 'claim-actions.ndjson')

def count(s, id):
    return sum((i['count'] for i in s['items'] if i['id'] == id))

def chest():
    g.act('object 57694 10423 115 1')
g.wait(lambda state:state.get('ready'),120)
g.interface(35913795)
s = g.wait(lambda s: s.get('ready') and 'Next: Claim reward' in s['journal'], 90)
if a.full_pack:
    g.act('close')
    g.act('walk 10439 116')
    s = g.wait(lambda s: s['position'][1:] == [10439, 116], 55)
    fish = next((n for n in s['npcs'] if n['id'] == 3317))
    g.act(f"npc {fish['index']} 1")
    g.wait(lambda s: len(s['items']) == 28 and sum(i['count'] > 0 for i in s['items']) >= 28, 110)
    chest()
    s = g.wait(lambda s: any(('Your reward is safe. Make room' in m for m in s['messages'])), 55)
    assert count(s, 995) == 0, 'Full-pack claim minted coins'
    assert 'First reward: unclaimed' in s['journal']
    g.act('capture' if a.native_frames else 'observe')
    for _ in range(2):
        s = g.observe()
        item = next((i for i in s['items'] if i['id'] == 317))
        g.act(f"item {item['slot']} 7")
        g.wait(lambda s: sum((i['count'] > 0 for i in s['items'])) <= 27 - _, 20)
before = g.observe()
coins = count(before, 995)
beers = count(before, 1917)
chest()
s = g.wait(lambda s: count(s, 995) == coins + a.coins and count(s, 1917) == beers + 1 and ('Done: Claim reward' in s['journal']), 55)
chest()
time.sleep(2)
s = g.observe()
assert count(s, 995) == coins + a.coins and count(s, 1917) == beers + 1, 'Duplicate chest payout'
g.act('capture' if a.native_frames else 'observe')
(a.output / 'claim-summary.json').write_text(json.dumps({'accepted': True, 'expected_coins': a.coins, 'state': s}, indent=2), encoding='utf8')
print('Durably acknowledged reward confirmed; duplicate search paid nothing', flush=True)
