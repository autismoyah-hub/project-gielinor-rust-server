import argparse, os, pathlib, json, time, threading, ctypes
from ctypes import wintypes
from tower_gateway import Gateway
p = argparse.ArgumentParser()
p.add_argument('--directory', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
if os.name != 'nt':
    raise SystemExit('Windows snapshot-sharing regression only')
p = a.directory
g = Gateway(p, a.output / 'gateway-lock-actions.ndjson')
s = g.wait(lambda s: s['ready'] and any((i['id'] == 1205 for i in s['equipment'])), 120)
g.act('close')
g.act('walk 10424 120')
g.wait(lambda s: s['ready'] and s['position'][1:] == [10424, 120], 55)
g.interface(35913795)
before = g.wait(lambda s: any((i['slot'] == 0 and i['id'] == 1351 and ('Wield' in i['operations']) for i in s['items'])), 40)
k = ctypes.WinDLL('kernel32', use_last_error=True)
k.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD, wintypes.LPVOID, wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
k.CreateFileW.restype = wintypes.HANDLE
k.CloseHandle.argtypes = [wintypes.HANDLE]
k.CloseHandle.restype = wintypes.BOOL
h = k.CreateFileW(str((p / 'state.json').resolve()), 2147483648, 1, None, 3, 128, None)
assert h not in (None, ctypes.c_void_p(-1).value), ctypes.get_last_error()
timer = threading.Timer(2, lambda: k.CloseHandle(h))
timer.start()
started = time.monotonic()
try:
    ack = g.act('item 0 3', timeout=12)
finally:
    timer.join(3)
elapsed = time.monotonic() - started
assert elapsed >= 1.8, 'Snapshot lock did not delay acknowledgement as expected'
after = g.wait(lambda s: s['ready'] and any((i['id'] == 1351 for i in s['equipment'])), 20)
assert after['hp'] > 0
s = g.wait(lambda s: any((i['slot'] == 0 and i['id'] == 1205 and ('Wield' in i['operations']) for i in s['items'])), 20)
g.act('item 0 3')
restored = g.wait(lambda s: any((i['id'] == 1205 for i in s['equipment'])), 20)
(a.output / 'gateway-lock-summary.json').write_text(json.dumps({'accepted': True, 'lock_seconds': 2, 'ack_elapsed_seconds': elapsed, 'before': before, 'after_locked_wield': after, 'restored': restored}, indent=2), encoding='utf8')
print('Native client survived a forced two-second snapshot sharing violation; normal weapon swap acknowledged and restored', round(elapsed, 2), flush=True)
