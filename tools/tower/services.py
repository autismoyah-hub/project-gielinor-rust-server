"""Drive Cinderworks services through normal client packets, with outcome assertions."""
import argparse, json, pathlib, time, subprocess, sys
from tower_gateway import Gateway
p = argparse.ArgumentParser()
p.add_argument('--directory', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
p.add_argument('--onboard', action='store_true')
p.add_argument('--gate-check', action='store_true')
p.add_argument('--stage', choices=['all', 'kitchen', 'altar'], default='all')
p.add_argument('--order', choices=['workshop-first','kitchen-first'], default='workshop-first')
p.add_argument('--scene-checks', action='store_true')
p.add_argument('--native-frames', action='store_true')
p.add_argument('--coolant-lesson', action='store_true')
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
g = Gateway(a.directory, a.output / 'services-actions.ndjson')

def count(s, item):
    return sum((i['count'] for i in s['items'] if i['id'] == item))

def walk(x, z):
    g.act(f'walk {x} {z}')
    return g.wait(lambda s: s['position'][1:] == [x, z], 55)

def obj(id, x, z):
    return g.act(f'object {id} {x} {z} 1')

def itemop(item, label):
    s = g.wait(lambda state:any(i['id']==item and any(v.lower()==label.lower() for v in i['operations']) for i in state['items']),40)
    i = next((i for i in s['items'] if i['id'] == item and any(v.lower()==label.lower() for v in i['operations'])))
    op = next((n + 1 for n, v in enumerate(i['operations']) if v.lower() == label.lower()))
    g.act(f"item {i['slot']} {op}")

def msg(text, timeout=50):
    return g.wait(lambda s: any((text in m for m in s['messages'])), timeout)
s = g.wait(lambda s: s.get('ready') and s.get('tick', 0) > 0, 120)
if a.onboard:
    g.wait(lambda s: 679 in s['modals'], 55)
    g.interface(44499018)
    g.wait(lambda s: 929 in s['modals'], 55)
    time.sleep(1)
    g.interface(60882951)
    msg('Welcome to the Cinderworks')
g.act('close')
g.interface(35913795)
before_plaque = g.observe()
obj(909, 10422, 115)
if 'Done: Claim reward' in before_plaque['journal']:
    # Existing tools cannot acknowledge a replay reset. Wait for its journal.
    g.wait(lambda s: s['journal'] != before_plaque['journal'] and 'Next: Workshop:' in s['journal'], 55)
g.wait(lambda s: count(s, 1265) > 0, 40)
if a.gate_check:
    walk(10439, 136)
    s = g.observe()
    xp = s['xp'][:]
    target = next((n for n in s['npcs'] if n['id'] == 16300))
    g.act(f"npc {target['index']} 2")
    s = msg('Restore the workshop, kitchen and altar')
    assert s['xp'] == xp, 'Denied guardian must not grant combat XP'
    g.act('walk 10424 120')
    g.wait(lambda s: s['position'][1:] == [10424, 120], 55)
def workshop():
    if count(g.observe(), 2349) == 0:
        if count(g.observe(), 438) == 0:
            obj(11360, 10402, 118)
            g.wait(lambda s: count(s, 438) > 0, 55)
        if count(g.observe(), 436) == 0:
            obj(10943, 10412, 118)
            g.wait(lambda s: count(s, 436) > 0, 55)
        obj(37947, 10402, 114)
        g.wait(lambda s: 270 in s['modals'], 55)
        time.sleep(1)
        g.interface(17694735)
        g.wait(lambda s: count(s, 2349) > 0, 55)
    g.act('close')
    obj(2097, 10405, 114)
    g.wait(lambda s: 312 in s['modals'], 55)
    time.sleep(1)
    old_daggers=count(g.observe(),1205)
    g.interface(20447241)
    g.wait(lambda s: count(s, 1205) > old_daggers and 'Done: Workshop:' in s['journal'], 55)
    g.act('close')
    itemop(1205, 'Wield')
    g.wait(lambda s: any((i['id'] == 1205 for i in s['equipment'])), 30)
    print('Workshop forged and equipped', flush=True)
def kitchen():
    if count(g.observe(), 317) == 0:
        walk(10439, 116)
        s = g.observe()
        fish = next((n for n in s['npcs'] if n['id'] == 3317))
        g.act(f"npc {fish['index']} 1")
        g.wait(lambda s: count(s, 317) >= 18 or sum(i['count'] > 0 for i in s['items']) >= 28, 110)
    s = g.wait(lambda state:any(i['id']==317 and i['widget'] is not None for i in state['items']),40)
    raw = next((i for i in s['items'] if i['id'] == 317 and i['widget'] is not None))
    g.act(f"select-item {raw['slot']}")
    g.act('use-object 9736 10435 117')
    g.wait(lambda s: count(s, 317) == 0 and count(s, 315) >= 12, 110)
    g.act('walk 10428 120')
    g.wait(lambda s: s['position'][1:] == [10428, 120], 55)
    print('Kitchen cooked supplies', flush=True)
def altar():
    walk(10437, 106)
    s = g.observe()
    cow = min((n for n in s['npcs'] if n['id'] == 1173), key=lambda n: abs(n['x'] - 10437) + abs(n['z'] - 106))
    index = cow['index']
    last = (cow['x'], cow['z'])
    g.act(f'npc {index} 2')
    end = time.monotonic() + 120
    eat_at = 0
    while time.monotonic() < end:
        s = g.observe()
        cow = next((n for n in s['npcs'] if n['index'] == index and n['id'] == 1173), None)
        if cow:
            last = (cow['x'], cow['z'])
        bones = [b for b in s['ground'] if b['id'] == 526]
        if bones and cow is None:
            if sum(i['count'] > 0 for i in s['items']) >= 28:
                if s['hp'] < s['base'][3]:
                    g.eat(s)
                else:
                    food = next(i for i in s['items'] if i['id']==315)
                    g.act(f"item {food['slot']} 7")
                g.wait(lambda state: sum(i['count'] > 0 for i in state['items']) < 28, 15)
                time.sleep(1.4)
            b = min(bones, key=lambda b: abs(b['x'] - last[0]) + abs(b['z'] - last[1]))
            g.act(f"ground 526 {b['x']} {b['z']} 3")
            g.wait(lambda s: count(s, 526) > 0, 35)
            break
        if 0 < s['hp'] <= 6 and time.monotonic() - eat_at > 2:
            g.eat(s)
            eat_at = time.monotonic()
            g.act(f'npc {index} 2')
        if s['hp'] <= 0:
            raise RuntimeError('Creature combat failed')
        time.sleep(0.2)
    else:
        raise TimeoutError('Creature did not yield observable bones')
    s = g.wait(lambda state:any(i['id']==526 and i['widget'] is not None for i in state['items']),40)
    bone = next((i for i in s['items'] if i['id'] == 526 and i['widget'] is not None))
    g.act(f"select-item {bone['slot']}")
    g.act('use-object 409 10428 116')
    s = g.wait(lambda s: count(s, 526) == 0 and 'Done: Altar:' in s['journal'], 55)

def check_scene(label):
    s=g.observe();j=s['journal']
    forge='Done: Workshop:' in j;water='Done: Kitchen:' in j;altar_done='Done: Altar:' in j
    ids=[18526 if forge else 18525,18511 if water else 18512,16811 if altar_done else 16810,
         16811 if 'Done: Quench Ash Warden' in j or 'First reward: claimed' in j else 16810,
         16810 if 'Done: Quench Ash Warden' in j else 16811]
    command=[sys.executable,str(pathlib.Path(__file__).with_name('restoration.py')),'--directory',str(a.directory),'--output',str(a.output),'--label',label,'--expected',','.join(map(str,ids))]
    if a.native_frames:command.append('--native-frames')
    subprocess.run(command,check=True)

if a.coolant_lesson:
    walk(10438,116)
    before=g.observe();uid=max((e['uid'] for e in before['message_events']),default=0)
    obj(30124,10437,116)
    after=g.wait(lambda s:any(e['uid']>uid and 'Cold water draws the heat' in e['text'] for e in s['message_events']),35)
    pack=lambda s:[(i['slot'],i['id'],i['count']) for i in s['items'] if i['count']>0]
    assert pack(before)==pack(after) and before['xp']==after['xp'] and after['hp']>=before['hp']
    (a.output/'coolant-lesson-summary.json').write_text(json.dumps({'accepted':True,'before':before,'after':after},indent=2),encoding='utf8')
    print('Ordinary Search teaches coolant without damage, item use, XP or unlocks',flush=True)
if a.scene_checks:check_scene('before-services')
order=['workshop','kitchen','altar'] if a.order=='workshop-first' else ['kitchen','workshop','altar']
stages=order if a.stage=='all' else ['kitchen','altar'] if a.stage=='kitchen' else ['altar']
for stage in stages:
    g.interface(35913795)
    {'workshop':workshop,'kitchen':kitchen,'altar':altar}[stage]()
    if a.scene_checks:check_scene('after-'+stage)
s=g.wait(lambda s:s['ready'] and all(label in s['journal'] for label in ['Done: Workshop:','Done: Kitchen:','Done: Altar:']),30)
(a.output/'services-summary.json').write_text(json.dumps({'accepted':True,'order':stages,'state':s},indent=2),encoding='utf8')
print('All three services confirmed through authoritative journal',flush=True)
