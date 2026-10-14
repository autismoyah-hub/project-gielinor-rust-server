"""Prepare an offline clone at CharCreate; this is not new-account creation."""
import json


def prepare_creator_resume(db, step_id, completed_id):
    row = db.execute("""SELECT c.id,c.world_id,c.varps FROM characters c
        JOIN accounts a ON a.id=c.account_id
        WHERE a.login_username='integration239' AND c.realm_id=1""").fetchone()
    if row is None or row[1] is not None:
        raise ValueError('Creator resume fixture must contain an offline account')
    values = json.loads(row[2])
    if not isinstance(values, dict) or step_id == completed_id:
        raise ValueError('Invalid creator variable identities or storage')
    values[str(step_id)] = 0
    values[str(completed_id)] = 0
    # Tutorial spawn from the unchanged V7/V8 migration: 0_26_95_22_35.
    db.execute('UPDATE characters SET x=1686,z=6115,level=0,varps=? WHERE id=?',
               (json.dumps(values, separators=(',', ':')), row[0]))
