from tower_gateway import Gateway
import argparse, time, pathlib, json
p = argparse.ArgumentParser()
p.add_argument('boss', choices=['imp', 'warden'])
p.add_argument('--directory', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
p.add_argument('--native-frames', action='store_true')
p.add_argument('--timeout', type=float, default=360)
p.add_argument('--label', default='accepted')
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
g = Gateway(a.directory, a.output / f'{a.boss}-{a.label}-actions.ndjson')
kind = 16300 if a.boss == 'imp' else 16295
destination = [10439, 136] if a.boss == 'imp' else [10423, 136]
bounds = (10434, 10445, 130, 141) if a.boss == 'imp' else (10418, 10430, 131, 141)
s = g.wait(lambda s: s.get('ready') and s.get('tick', 0) > 0 and (s.get('hp', 0) > 0), 120)
if s.get('equipment') is not None and (not any((item['id'] in (1205, 1321) for item in s['equipment']))):
    raise RuntimeError('A forged bronze weapon must be worn before the guardian run')
g.act('close')
g.interface(35913795)
g.act('camera 2048 0')
g.act('walk ' + ' '.join(map(str, destination)))
s = g.wait(lambda s: s['position'][1:] == destination, 55)
s = g.wait(lambda state:any(n['id']==kind for n in state['npcs']),30)
target = next((n for n in s['npcs'] if n['id'] == kind), None)
if target is None:
    raise RuntimeError('Guardian not present in the observed court')
index = target['index']
g.act(f'npc {index} 2')
print('Started', a.boss, 'through normal Attack action', index, flush=True)
end = time.monotonic() + a.timeout
last_report = 0
last_eat = 0
hold_until = -1
need_attack = False
last_walk = None
last_walk_at = 0
captures = []
phases = set()
warnings = 0
last_quench = -1000
quenching_until = 0
last_quench_message = None
last_burst = -1
wave_active = False
attempted = False
quench_ready = 0
while time.monotonic() < end:
    s = g.observe()
    if not s['ready']:
        time.sleep(.05)
        continue
    now = time.monotonic()
    tile = tuple(s['position'][1:])
    messages = s.get('messages', [])
    victory = any((('The Ash Imp falls' if a.boss == 'imp' else 'The Warden is quenched') in text for text in messages))
    if victory:
        g.act('walk 10424 128')
        g.wait(lambda state: state['position'][1:] == [10424, 128], 55)
        shot = g.act('capture' if a.native_frames else 'observe')
        captures.append(shot['sequence'])
        result = {'boss': a.boss, 'accepted': True, 'state': s, 'mode': 'native' if a.native_frames else 'headless', 'frame_sequences': captures if a.native_frames else [], 'observation_sequences': captures, 'evasion_requests': warnings, 'phases_seen': sorted(phases)}
        (a.output / f'{a.boss}-{a.label}-summary.json').write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding='utf-8')
        print('Authoritative victory confirmed', a.boss, 'HP', s['hp'], 'captures', captures, flush=True)
        break
    if not (bounds[0] <= tile[0] <= bounds[1] and bounds[2] <= tile[1] <= bounds[3]) or s['hp'] <= 0:
        raise RuntimeError('Encounter failed or player left the court; no victory accepted')
    target = next((n for n in s['npcs'] if n['index'] == index and n['id'] == kind), None)
    if target is None:
        time.sleep(0.2)
        continue
    for text in messages:
        phase = (3 if 'The Warden enters its final phase' in text else 2 if 'The Warden enters phase two' in text else None) if a.boss == 'warden' else 2 if 'The Imp scatters its heat' in text else None
        if phase and phase not in phases:
            phases.add(phase)
            print('Observed phase', phase, flush=True)
    burst = max((x['start'] for x in s['graphics'] if x['id'] == 78 and bounds[0] <= x['x'] <= bounds[1] and bounds[2] <= x['z'] <= bounds[3]), default=last_burst)
    if burst > last_burst:
        last_burst = burst
        hold_until = s['tick'] - 1
        wave_active = False
        need_attack = True
    signature = tuple(messages)
    if messages and messages[0].startswith('Coolant quenches the charge!') and (signature != last_quench_message):
        last_quench_message = signature
        last_burst = max(last_burst, s['cycle'])
        hold_until = s['tick'] - 1
        wave_active = False
        need_attack = True
        quench_ready = s['tick'] + 9
        print('Authoritative coolant interruption confirmed', flush=True)
    hot = {(x['x'], x['z']) for x in s['graphics'] if x['id'] == 86 and x['start'] > last_burst and bounds[0] <= x['x'] <= bounds[1] and bounds[2] <= x['z'] <= bounds[3]}
    if hot:
        if not wave_active:
            wave_active = True
            attempted = False
        hold_until = max(hold_until, s['tick'] + 2)
        if a.boss == 'warden' and (not attempted) and (s['tick'] >= quench_ready) and (now >= quenching_until):
            barrel = min([(10421, 135), (10428, 135)], key=lambda t: abs(t[0] - tile[0]) + abs(t[1] - tile[1]))
            g.act(f'object 362 {barrel[0]} {barrel[1]} 1')
            last_quench = s['tick']
            quenching_until = now + 2.1
            need_attack = True
            attempted = True
        elif tile in hot and now >= quenching_until:
            candidates = [(tile[0] + dx, tile[1] + dz) for dx, dz in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)]]
            candidates = [t for t in candidates if bounds[0] <= t[0] <= bounds[1] and bounds[2] <= t[1] <= bounds[3] and (t not in hot) and (t != (target['x'], target['z']))]
            if not candidates:
                raise RuntimeError('No observed safe adjacent tile in the encounter pattern')
            dest = min(candidates, key=lambda t: ((t[0] // 8, t[1] // 8) != (tile[0] // 8, tile[1] // 8), abs(t[0] - target['x']) + abs(t[1] - target['z'])))
            if dest != last_walk or now - last_walk_at > 0.65:
                g.act(f'walk {dest[0]} {dest[1]}')
                last_walk = dest
                last_walk_at = now
                need_attack = True
                warnings += 1
        if not captures or (2 in phases and len(captures) < 2) or (3 in phases and len(captures) < 3):
            shot = g.act('capture' if a.native_frames else 'observe')
            captures.append(shot['sequence'])
            print('Recorded telegraph observation', shot['sequence'], flush=True)
    else:
        quenching_until = 0
    if 0 < s['hp'] <= 6 and now - last_eat > 1.9 and (not (tile in hot and now >= quenching_until)):
        g.eat(s)
        last_eat = now
        need_attack = True
        print('Requested food at HP', s['hp'], flush=True)
    safe_imp_melee = a.boss == 'imp' and tile not in hot and max(abs(tile[0]-target['x']),abs(tile[1]-target['z'])) == 1
    if need_attack and ((not hot and s['tick'] > hold_until) or safe_imp_melee) and now-last_eat > 1.2:
        g.act(f'npc {index} 2')
        need_attack = False
    if now - last_report >= 15:
        print('Progress', a.boss, 'HP', s['hp'], 'tile', tile, 'evasive walk requests', warnings, 'food', sum((i['count'] for i in s['items'] if i['id'] == 315)), 'latest', messages[:2], flush=True)
        last_report = now
    time.sleep(0.16)
else:
    raise TimeoutError('No authoritative guardian victory within the bounded acceptance run')
