"""Restock a fresh unpaid Warden run through normal fishing, cooking and Eat actions.

Run from the Tower workspace with --directory and --output, after the Imp.
Ordinary walking retries keep the original 55-second bound. No account, item,
statistic, timer or encounter state is written directly.
"""
import sys, pathlib, json, argparse
sys.path.insert(0, str(pathlib.Path('rust-server/tools/tower').resolve()))
from tower_gateway import Gateway
p = argparse.ArgumentParser()
p.add_argument('--directory', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
a = p.parse_args()
a.output.mkdir(exist_ok=True)
g = Gateway(a.directory, a.output / 'restock-actions.ndjson')
before = g.wait(lambda s: s['ready'], 30)
count = lambda s, id: sum((i['count'] for i in s['items'] if i['id'] == id))
pack = lambda s: sorted(((i['slot'], i['id'], i['count']) for i in s['items'] if i['count'] > 0 and i['id'] != 315))

def walk(x, z):
    import time
    end = time.monotonic() + 55
    retry = 0
    while time.monotonic() < end:
        s = g.observe()
        if s['ready'] and s['position'][1:] == [x, z]:
            return s
        if s['ready'] and time.monotonic() >= retry:
            g.act(f'walk {x} {z}')
            retry = time.monotonic() + 4
        time.sleep(0.2)
    raise TimeoutError('Ordinary walking retries did not reach preparation stance')
assert 'Next: Quench Ash Warden' in before['journal'] and 'First reward: unclaimed' in before['journal']
g.act('close')
g.interface(35913795)
while True:
    s = g.observe()
    if s['hp'] >= s['base'][3]:
        break
    prior = count(s, 315)
    g.eat(s)
    g.wait(lambda s: count(s, 315) < prior, 15)
walk(10438, 116)
for batch in range(5):
    s = g.observe()
    if count(s, 315) == 22:
        break
    assert count(s, 315) < 22
    fish = min((n for n in s['npcs'] if n['id'] == 3317), key=lambda n: abs(n['x'] - 10438) + abs(n['z'] - 116))
    g.act(f"npc {fish['index']} 1")
    g.wait(lambda s: sum((i['count'] > 0 for i in s['items'])) >= 28, 160)
    s = g.observe()
    raw = next((i for i in s['items'] if i['id'] == 317 and i.get('widget') is not None))
    g.act(f"select-item {raw['slot']}")
    g.act('use-object 9736 10435 117')
    g.wait(lambda s: count(s, 317) == 0, 160)
else:
    raise RuntimeError('Ordinary restock did not complete')
after = walk(10424, 120)
assert count(after, 315) == 22 and after['journal'] == before['journal'] and (after['equipment'] == before['equipment']) and (pack(after) == pack(before))
assert all((x == y for i, (x, y) in enumerate(zip(before['xp'], after['xp'])) if i not in (7, 10)))
(a.output / 'restock-summary.json').write_text(json.dumps({'accepted': True, 'ordinary_fishing_and_cooking': True, 'before': before, 'after': after}, indent=2))
print('Normal preparation: full health and 22 caught/cooked shrimp', flush=True)
