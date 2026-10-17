from tower_gateway import Gateway
import argparse, time, pathlib, json
p = argparse.ArgumentParser()
p.add_argument('boss', choices=['imp', 'warden'])
p.add_argument('--directory', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
p.add_argument('--native-frames', action='store_true')
p.add_argument('--style', choices=['aggressive','accurate'], default='aggressive')
p.add_argument('--require-coolant', action='store_true', help='Demonstrate an acknowledged adjacent coolant interruption before resuming melee')
p.add_argument('--timeout', type=float, default=360)
p.add_argument('--label', default='accepted')
a = p.parse_args()
if a.require_coolant and a.boss!='warden':
    raise ValueError('Coolant belongs to the Warden encounter')
a.output.mkdir(parents=True, exist_ok=True)
g = Gateway(a.directory, a.output / f'{a.boss}-{a.label}-actions.ndjson')
kind = 16300 if a.boss == 'imp' else 2075
destination = [10439, 136] if a.boss == 'imp' else [10423, 136]
bounds = (10436, 10443, 132, 139) if a.boss == 'imp' else (10418, 10430, 131, 141)
coolant=[(10421,135),(10421,138),(10428,135),(10428,138)]
s = g.wait(lambda s: s.get('ready') and s.get('tick', 0) > 0 and (s.get('hp', 0) > 0), 120)
if s.get('equipment') is not None and (not any((item['id'] in (1205, 1321) for item in s['equipment']))):
    raise RuntimeError('A forged bronze weapon must be worn before the guardian run')
if 'message_events' not in s:
    raise RuntimeError('Use the current validation client with normal chat UID observations')
g.act('close')
# Select the displayed aggressive dagger style through normal UI operations.
# Strength is earned from real combat; no levels, damage or XP are injected.
g.interface(35913792)
g.interface(38862854 if a.style=='accurate' else 38862858)
g.wait(lambda s:s['ready'],20)
g.interface(35913795)
g.act('camera 2048 0')
g.act('walk ' + ' '.join(map(str, destination)))
s = g.wait(lambda s: s['position'][1:] == destination, 55)
s = g.wait(lambda state:any(n['id']==kind for n in state['npcs']),30)
target = next((n for n in s['npcs'] if n['id'] == kind), None)
if target is None:
    raise RuntimeError('Guardian not present in the observed court')
index = target['index']
fight_start_uid=max((e['uid'] for e in s.get('message_events',[])),default=-1)
last_quench_uid=fight_start_uid
coolant_events=[]
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
feedback_captures=[]
phases = set()
warnings = 0
last_quench = -1000
quenching_until = 0
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
    fresh_events=[e for e in s.get('message_events',[]) if e['uid']>fight_start_uid]
    outcome_texts=[e['text'] for e in fresh_events]
    victory = any((('The Ash Imp falls' if a.boss == 'imp' else 'The Warden is quenched') in text for text in outcome_texts))
    if victory:
        if a.require_coolant and not coolant_events:
            raise RuntimeError('Victory did not exercise the required coolant mechanic')
        if a.native_frames:
            view=g.act('capture');feedback_captures.append({'reason':'victory in arena','sequence':view['sequence']})
        g.act('walk 10424 128')
        g.wait(lambda state: state['position'][1:] == [10424, 128], 55)
        shot = g.act('capture' if a.native_frames else 'observe')
        captures.append(shot['sequence'])
        result = {'boss': a.boss, 'accepted': True, 'state': s, 'mode': 'native' if a.native_frames else 'headless', 'frame_sequences': captures if a.native_frames else [], 'observation_sequences': captures, 'evasion_requests': warnings, 'phases_seen': sorted(phases), 'coolant_ack_uids':coolant_events, 'coolant_required':a.require_coolant, 'style':a.style, 'feedback_captures':feedback_captures}
        (a.output / f'{a.boss}-{a.label}-summary.json').write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding='utf-8')
        print('Authoritative victory confirmed', a.boss, 'HP', s['hp'], 'captures', captures, flush=True)
        break
    if not (bounds[0] <= tile[0] <= bounds[1] and bounds[2] <= tile[1] <= bounds[3]) or s['hp'] <= 0:
        raise RuntimeError('Encounter failed or player left the court; no victory accepted')
    target = next((n for n in s['npcs'] if n['index'] == index and n['id'] == kind), None)
    if target is None:
        time.sleep(0.2)
        continue
    for text in outcome_texts:
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
    quench_events=[e for e in fresh_events if e['uid']>last_quench_uid and e['text'].startswith('Coolant quenches the charge!')]
    if quench_events:
        event=max(quench_events,key=lambda e:e['uid'])
        last_quench_uid=event['uid'];coolant_events.append(event['uid'])
        # Chat timestamps use the UI clock; ground graphics use actor time.
        # Only compare graphics with the observed actor cycle.
        last_burst=max(last_burst,s['cycle'])
        hold_until=s['tick']-1;wave_active=False;need_attack=True;quench_ready=s['tick']+9
        print('Authoritative coolant interruption confirmed',flush=True)
        if a.native_frames and not feedback_captures:
            view=g.act('capture');feedback_captures.append({'reason':'coolant steam and flinch','sequence':view['sequence']})
    hot = {(x['x'], x['z']) for x in s['graphics'] if x['id'] == 86 and x['start'] > last_burst and bounds[0] <= x['x'] <= bounds[1] and bounds[2] <= x['z'] <= bounds[3]}
    if hot:
        if not wave_active:
            wave_active = True
            attempted = False
        hold_until = max(hold_until, s['tick'] + 2)
        # Evade an observed dangerous tile first. Chase no remote interaction
        # through a four-tick warning: quench only from a safe adjacent tile.
        adjacent_barrel=min(coolant,key=lambda t:abs(t[0]-tile[0])+abs(t[1]-tile[1]))
        instant_quench=a.boss=='warden' and a.require_coolant and not coolant_events and not attempted and s['tick']>=quench_ready and now>=quenching_until and abs(adjacent_barrel[0]-tile[0])+abs(adjacent_barrel[1]-tile[1])==1
        if instant_quench:
            g.act(f'object 30124 {adjacent_barrel[0]} {adjacent_barrel[1]} 1')
            last_quench=s['tick'];quenching_until=now+1.0;need_attack=True;attempted=True
        elif tile in hot:
            candidates = [(tile[0] + dx, tile[1] + dz) for dx, dz in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)]]
            size=2 if a.boss=='warden' else 1
            occupied={(target['x']+dx,target['z']+dz) for dx in range(size) for dz in range(size)}
            if a.boss=='warden':occupied.update(coolant)
            candidates = [t for t in candidates if bounds[0] <= t[0] <= bounds[1] and bounds[2] <= t[1] <= bounds[3] and t not in hot and t not in occupied and not (t[0]!=tile[0] and t[1]!=tile[1] and ((t[0],tile[1]) in occupied or (tile[0],t[1]) in occupied))]
            if not candidates:
                raise RuntimeError('No observed safe adjacent tile in the encounter pattern')
            dest = min(candidates, key=lambda t: ((a.boss=='warden' and min(abs(t[0]-b[0])+abs(t[1]-b[1]) for b in coolant)!=1), (t[0] // 8, t[1] // 8) != (tile[0] // 8, tile[1] // 8), abs(t[0] - target['x']) + abs(t[1] - target['z'])))
            if dest != last_walk or now - last_walk_at > 0.65:
                g.act(f'walk {dest[0]} {dest[1]}')
                last_walk = dest
                last_walk_at = now
                need_attack = True
                warnings += 1
        elif a.boss == 'warden' and not attempted and s['tick']>=quench_ready and now>=quenching_until:
            barrel=min(coolant,key=lambda t:abs(t[0]-tile[0])+abs(t[1]-tile[1]))
            if abs(barrel[0]-tile[0])+abs(barrel[1]-tile[1])==1:
                g.act(f'object 30124 {barrel[0]} {barrel[1]} 1')
                last_quench=s['tick'];quenching_until=now+2.1;need_attack=True;attempted=True
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
    if need_attack and (not a.require_coolant or coolant_events) and ((not hot and s['tick'] > hold_until) or safe_imp_melee) and now-last_eat > 1.2:
        g.act(f'npc {index} 2')
        need_attack = False
    if now - last_report >= 15:
        print('Progress', a.boss, 'HP', s['hp'], 'tile', tile, 'evasive walk requests', warnings, 'food', sum((i['count'] for i in s['items'] if i['id'] == 315)), 'latest', messages[:2], flush=True)
        last_report = now
    time.sleep(0.16)
else:
    raise TimeoutError('No authoritative guardian victory within the bounded acceptance run')
