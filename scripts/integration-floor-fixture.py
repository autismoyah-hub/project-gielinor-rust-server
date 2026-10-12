"""Create an offline disposable floor-render fixture, never a teleport test.

Preserves the inventory A/B account by backing it up to a separate state directory.
"""
import json
from pathlib import Path
import sqlite3

root = Path(__file__).resolve().parent.parent
source = root / 'work/state/integration-b/accounts.sqlite'
destination = root / 'work/state/integration-floor'
if destination.exists():
    raise ValueError('floor fixture already exists; refusing to replace it')
with sqlite3.connect(source.as_uri() + '?mode=ro', uri=True) as original:
    character = original.execute("""SELECT c.id,c.world_id,c.x,c.z,c.level
        FROM characters c JOIN accounts a ON a.id=c.account_id
        WHERE a.login_username='integration239' AND c.realm_id=1""").fetchone()
    if character is None or character[1] is not None:
        raise ValueError('source integration account must exist and be logged out')
    destination.mkdir()
    with sqlite3.connect(destination / 'accounts.sqlite') as copied:
        original.backup(copied)
        copied.execute('UPDATE characters SET x=10424,z=120,level=0 WHERE id=?', (character[0],))
record = {'fixture_setup_only': True, 'scenario': 'constructed-floor-render',
          'source': str(source), 'database': str(destination / 'accounts.sqlite'),
          'position_before': character[2:], 'fixture_position': [10424, 120, 0],
          'login_relocation_verified': False}
(destination / 'fixture.json').write_text(json.dumps(record, indent=2) + '\n')
print('Created separate floor fixture; no gameplay relocation claimed')
