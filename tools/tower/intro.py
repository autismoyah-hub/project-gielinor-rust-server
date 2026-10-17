"""Check normal onboarding and the authored introduction before a run begins."""
import argparse, pathlib, json, time
from tower_gateway import Gateway
p = argparse.ArgumentParser()
p.add_argument('--directory', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
g = Gateway(a.directory, a.output / 'intro-actions.ndjson')
g.wait(lambda s: s['ready'] and 679 in s['modals'], 120)
g.interface(44499018)
g.wait(lambda s: s['ready'] and 929 in s['modals'], 55)
g.interface(60882951)
g.wait(lambda s: s['ready'] and (not s['modals']) and any(('Welcome to the Cinderworks' in m for m in s['messages'])), 55)
g.interface(35913794)
g.interface(41222162)
s = g.wait(lambda s: s['ready'] and 'Read the sanctuary plaque' in s['journal'], 55)
assert not any((i['count'] > 0 for i in s['items'])), 'Unstarted run granted items'
assert 'Cinderworks' in s['journal'] and 'Hides' not in s['journal']
assert 'run 1' not in s['journal']
(a.output / 'intro-summary.json').write_text(json.dumps({'accepted': True, 'state': s}, indent=2), encoding='utf8')
print('Normal onboarding shows Cinderworks introduction before plaque, with no items or implicit run', flush=True)
