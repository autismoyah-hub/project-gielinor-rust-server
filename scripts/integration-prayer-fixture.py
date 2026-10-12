"""Explicitly drain Prayer in the logged-out disposable floor fixture for an altar test.

This is test setup, not evidence that native prayer activation or draining works.
Inventory, XP and base levels are preserved. A SQLite backup and before/after record
are created before changing the one visible level.
"""
import json
from pathlib import Path
import sqlite3
import sys

root = Path(__file__).resolve().parent.parent
record = (root / "work/integration" / sys.argv[1]).resolve()
if record.parent != (root / "work/integration").resolve() or record.exists():
    raise ValueError("use a new record name within the isolated integration directory")
database = root / "work/state/integration-floor/accounts.sqlite"
backup = record.with_suffix(".sqlite")
if backup.exists():
    raise ValueError("fixture backup already exists")
db = sqlite3.connect(database.as_uri() + "?mode=rw", uri=True)
row = db.execute("""SELECT c.id,c.world_id,s.vis_level,s.base_level,s.fine_xp
    FROM characters c JOIN accounts a ON a.id=c.account_id
    JOIN stats s ON s.character_id=c.id
    WHERE a.login_username='integration239' AND c.realm_id=1 AND s.stat_id=5""").fetchone()
if row is None or row[1] is not None or row[2] <= 0 or row[3] <= 0:
    raise ValueError("expected a logged-out floor account with positive Prayer")
with sqlite3.connect(backup) as copy:
    db.backup(copy)
with db:
    changed = db.execute("""UPDATE stats SET vis_level=0
        WHERE character_id=? AND stat_id=5 AND vis_level=?
        AND EXISTS(SELECT 1 FROM characters WHERE id=? AND world_id IS NULL)""",
        (row[0], row[2], row[0])).rowcount
    if changed != 1:
        raise ValueError("fixture changed or account logged in")
db.close()
record.write_text(json.dumps({"purpose": "explicit altar test setup; not gameplay drain proof",
    "database": str(database), "backup": str(backup), "stat_id": 5,
    "visible_before": row[2], "visible_after": 0,
    "base_unchanged": row[3], "fine_xp_unchanged": row[4]}, indent=2) + "\n")
print(f"Prepared drained Prayer fixture; recorded {record.name}")
