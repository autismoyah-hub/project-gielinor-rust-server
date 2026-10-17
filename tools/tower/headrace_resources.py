import argparse, csv, json, pathlib, time
from tower_gateway import Gateway
p = argparse.ArgumentParser()
p.add_argument('--directory', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
g = Gateway(a.directory, a.output / 'resource-access-actions.ndjson')

def count(s, item):
    return sum((i['count'] for i in s['items'] if i['id'] == item))

def stable():
    return g.wait(lambda s: s['ready'] and s['tick'] > 0 and all((v > 0 for v in s['base'])), 120)

def walk(x, z):
    end = time.monotonic() + 60
    retry = 0
    while True:
        s = g.observe()
        if s['ready'] and s['position'][1:] == [x, z]:
            return s
        if time.monotonic() > end:
            raise TimeoutError('Ordinary resource approach blocked')
        if s['ready'] and time.monotonic() > retry:
            g.act(f'walk {x} {z}')
            retry = time.monotonic() + 4
        time.sleep(0.2)

def inv():
    g.interface(35913795)
    return stable()

def drop(item):
    s = inv()
    i = next((i for i in s['items'] if i['id'] == item and 'Drop' in i['operations']))
    before = count(s, item)
    g.act(f"item {i['slot']} 7")
    return g.wait(lambda s: count(s, item) < before, 20)
food = {315, 317, 323}
before = stable()
assert 'Done: Claim reward' in before['journal']
attempts = []
try:
    for attempt in range(3):
        s = inv()
        if sum((i['count'] > 0 for i in s['items'])) >= 28:
            drop(323 if count(s, 323) else 315)
            s = stable()
        old_raw = count(s, 317)
        old_fish_xp = s['xp'][10]
        walk(10439, 116)
        s = stable()
        fish = next((n for n in s['npcs'] if n['id'] == 3317))
        g.act(f"npc {fish['index']} 1")
        fished = g.wait(lambda s: s['ready'] and count(s, 317) > old_raw and (s['xp'][10] > old_fish_xp), 90)
        walk(10438, 116)
        s = inv()
        raw = next((i for i in s['items'] if i['id'] == 317 and i['count'] > 0))
        old_cooked = count(s, 315)
        old_cook_xp = s['xp'][7]
        g.act(f"select-item {raw['slot']}")
        g.act('use-object 9736 10435 117')
        cooked = g.wait(lambda s: s['ready'] and count(s, 317) == 0, 110)
        walk(10438, 116)
        s = stable()
        attempts.append({'attempt': attempt + 1, 'actual_raw_before': old_raw, 'actual_raw_after_fishing': count(fished, 317), 'fishing_XP_before': old_fish_xp, 'fishing_XP_after': s['xp'][10], 'cooked_before': old_cooked, 'cooked_after': count(s, 315), 'cooking_XP_before': old_cook_xp, 'cooking_XP_after': s['xp'][7]})
        if count(s, 315) > old_cooked and s['xp'][7] > old_cook_xp:
            break
    else:
        raise RuntimeError('No successful ordinary shrimp cooking in three bounded attempts')
    lesson_before = stable()
    uid = max((e['uid'] for e in lesson_before['message_events']))
    g.act('object 30124 10437 116 1')
    lesson_after = g.wait(lambda s: s['ready'] and any((e['uid'] > uid and e['text'].startswith('Cold water draws the heat') for e in s['message_events'])), 40)
    pack = lambda s: sorted(((i['slot'], i['id'], i['count']) for i in s['items'] if i['count'] > 0))
    assert pack(lesson_before) == pack(lesson_after) and lesson_before['xp'] == lesson_after['xp'] and (lesson_before['equipment'] == lesson_after['equipment']) and (lesson_after['hp'] >= lesson_before['hp'])
    walk(10424, 120)
    after = stable()
    nonfood = lambda s: sorted(((i['slot'], i['id'], i['count']) for i in s['items'] if i['count'] > 0 and i['id'] not in food))
    assert nonfood(before) == nonfood(after) and before['equipment'] == after['equipment'] and (before['journal'] == after['journal'])
    assert all((before['xp'][i] == after['xp'][i] for i in range(25) if i not in (7, 10)))
    assert after['xp'][7] > before['xp'][7] and after['xp'][10] > before['xp'][10]
    (a.output / 'resource-access-summary.json').write_text(json.dumps({'accepted': True, 'scope': 'Actual normal net fishing/cooking and harmless coolant after headrace placement; only legitimate Fishing/CookingXP and food changes, exact other inventory/equipment/journal/currency retained.', 'attempts': attempts, 'fresh_coolant_reply_uids': [e['uid'] for e in lesson_after['message_events'] if e['uid'] > uid and e['text'].startswith('Cold water draws the heat')], 'before': before, 'after': after}, indent=2), encoding='utf8')
    print('Actual fishing/cooking/coolant access passed; only real earned skillXP and food changed', flush=True)
except Exception as error:
    try:
        walk(10424, 120)
    except Exception:
        pass
    (a.output / 'resource-access-failed.json').write_text(json.dumps({'accepted': False, 'reason': str(error), 'attempts': attempts, 'state': g.read()}, indent=2), encoding='utf8')
    raise
