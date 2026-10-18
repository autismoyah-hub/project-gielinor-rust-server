"""Independent durable-state and received-game-packet oracles for manual sinew."""
import argparse, hashlib, json, re
from pathlib import Path

p = argparse.ArgumentParser(); p.add_argument('--crash', action='store_true'); a = p.parse_args()
ROOT = Path(__file__).resolve().parents[1]
SHA = '5582c276d1657a0142151936e5a9fec022d7c938'
OUT = ROOT / 'docs/openrune/verification/independent' / ('deep-sinew-crash/crash-001' if a.crash else 'deep-sinew/native-001')
load = lambda name: json.loads((OUT / name).read_text(encoding='utf-8'))
initial = load('initial-state.json'); after = load('produce/after.json'); post = load('relogin/after.json')
slots = [4] if a.crash else [4, 26, 9, 17]
expected = [r.copy() for r in initial['inventory']]
if not a.crash:
    for slot in slots: next(r for r in expected if r[:2] == [93, slot])[2] = 9436
assert after['inventory'] == post['inventory'] == expected
assert load('relogin/before.json') == after
for x, y, z in zip(initial['stats'], after['stats'], post['stats']):
    assert x[0] == y[0] == z[0] and y == z
    if a.crash or x[0] != 12: assert x == y
    else: assert y == [12, 99, 99, x[3] + 3000]
for phase in ['produce', 'relogin']:
    m = load(phase + '/manifest.json')
    assert m['tested_sha'] == m['implementation_sha'] == SHA and m['status'] == 'PASS'
    assert m['resource_hashes_unchanged'] and m['client_exit'] == 0
    assert load(phase + '/after.json')['integrity_check'] == 'ok'
    if a.crash and phase == 'produce':
        assert m['server_exit'] != 0 and m['intentional_crash_injected'] and 'sinew' in m['fault_scope']
        assert not m['save_acknowledgement'] and not m['graceful_shutdown']
    else:
        assert m['server_exit'] == 0 and m['save_acknowledgement'] and m['graceful_shutdown'] and not m['forced_shutdown']
log = (OUT / 'produce/client.log').read_text(encoding='utf-8')
if a.crash:
    assert 'PG_CRASH_READY native manual sinew' in log and 'PG_EXPECTED_CRASH_DISCONNECT' in log
else:
    assert 'PG_CASE_PASS native sinew pending movement cancels without inventory/XP mutation' in log
    assert 'PG_CASE_PASS native manual sinew four selected credits' in log
uses = re.findall(r'PG_WIRE_TX tick=(\d+) opcode=27 payload=\[(.*?)\]', log)
assert len(uses) == len(slots) + (0 if a.crash else 1)
for index, (_, hexes) in enumerate(uses):
    b = bytes.fromhex(hexes.replace(',', '')); assert len(b) == 16
    beadd = lambda v: (v[0] << 8) | ((v[1] - 128) & 255)
    leadd = lambda v: ((v[0] - 128) & 255) | (v[1] << 8)
    pair = (beadd(b[4:6]), beadd(b[10:12]), int.from_bytes(b[8:10], 'little'), leadd(b[6:8]))
    j = index if a.crash else index - 1
    assert pair == ((0, 946, 4, 19665) if j == -1 else (0, 946, slots[j], 19665) if j % 2 == 0 else (slots[j], 19665, 0, 946))
assert len(re.findall('PG_REPLAY label="native stale selected sinew pair', log)) == len(slots)
assert not re.search(r'PG_WIRE_TX tick=\d+ opcode=115 ', log)
effects = []; seen = set(); sequence = None; xp_seen = set()
for line in log.splitlines():
    seq = re.fullmatch(r'PG_SEQUENCE tick=(\d+) sequence=10937 delay=(\d+)', line)
    if seq: sequence = (int(seq[1]), int(seq[2]))
    rx = re.fullmatch(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*)\]', line)
    if not rx: continue
    tick, op = int(rx[1]), int(rx[2]); b = bytes.fromhex(rx[3].replace(',', ''))
    if op == 46 and (b[2] - 128) & 255 == 12: xp_seen.add(int.from_bytes(b[5:7] + b[3:5], 'big'))
    if op != 18 or int.from_bytes(b[4:6], 'big') != 93: continue
    pos = 6
    while pos < len(b):
        slot = b[pos]; pos += 1
        if slot >= 128: slot = ((slot << 8) | b[pos]) - 32768; pos += 1
        item = int.from_bytes(b[pos:pos+2], 'big') - 1; pos += 2; count = 0
        if item >= 0:
            count = b[pos]; pos += 1
            if count == 255: count = int.from_bytes(b[pos:pos+4], 'big'); pos += 4
        if item == 9436 and slot not in seen:
            assert slot == slots[len(effects)] and count == 1 and sequence is not None
            assert tick - sequence[0] == 1 and sequence[1] == 0
            effects.append({'slot': slot, 'sequence': 10937, 'sequence_tick': sequence[0], 'credit_tick': tick}); seen.add(slot)
    assert pos == len(b)
assert len(effects) == len(slots)
for i in range(len(slots)): assert (130344311 + 750 * (i + 1)) // 10 in xp_seen
invs = {}; stats = {}
for op, hexes in re.findall(r'PG_RX tick=\d+ opcode=(\d+) payload=\[(.*?)\]', (OUT / 'relogin/client.log').read_text(encoding='utf-8')):
    op = int(op); b = bytes.fromhex(hexes.replace(',', ''))
    if op == 46: stats[(b[2] - 128) & 255] = (b[1], int.from_bytes(b[5:7] + b[3:5], 'big'))
    if op != 22: continue
    inv = int.from_bytes(b[4:6], 'big'); size = int.from_bytes(b[6:8], 'big'); pos = 8; items = []
    for slot in range(size):
        count = (128 - b[pos]) & 255; pos += 1
        if count == 255: c = b[pos:pos+4]; count = int.from_bytes(bytes([c[1], c[0], c[3], c[2]]), 'big'); pos += 4
        item = int.from_bytes(b[pos:pos+2], 'little') - 1; pos += 2
        if item >= 0: items.append([inv, slot, item, count, 0])
    assert pos == len(b); invs[inv] = items
assert invs[93] == expected
for skill, current, base, fine in post['stats']: assert stats[skill] == (current, fine // 10)
report = {'implementation_id': 'OR-MIG-0066', 'tested_sha': SHA, 'campaign_kind': 'historical', 'status': 'PASS', 'crash': a.crash, 'observed_outputs': len(slots), 'durable_outputs': 0 if a.crash else 4, 'observed_fine_award': 750 * len(slots), 'effects': effects,
    'scope': 'Independent received held-Use, inventory update, Stats and sequence decoding; selected-only manual credit, retained duplicate inputs/tools/full bag, stale refusal and no Make-X. Whole durable inventory/Stats match actual process restart and independently decoded native reconnect. ' + ('Owned server abrupt termination after one observed unsaved product restores prior whole state; no save acknowledgement.' if a.crash else 'Actual pending Walk cancels before award. Four manual actions alternate both held directions and save 3000 fineXP without automatic repeats.'),
    'limitations': ['Native rate500 only; other rate/current-level/modifier cases are separately covered in authoritative World tests.', 'Received sequence timing does not establish rendered frames or audio.', 'Recovery fault is unacknowledged state rollback, not an injection during SQL transaction commit.'],
    'evidence_sha256': {name: hashlib.sha256((OUT / name).read_bytes()).hexdigest() for name in ['produce/client.log', 'produce/manifest.json', 'relogin/client.log', 'relogin/manifest.json']}}
(OUT / 'independent-assessment.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps(report))
