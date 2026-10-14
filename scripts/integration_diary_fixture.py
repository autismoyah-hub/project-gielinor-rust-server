"""Explicit, isolated known-empty diary fixture; never infer legacy emptiness."""
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def prepare_rust(db):
    row = db.execute("""SELECT c.id,c.world_id FROM characters c JOIN accounts a ON a.id=c.account_id
        WHERE a.login_username='integration239' AND c.realm_id=1""").fetchone()
    if row is None or row[1] is not None:
        raise ValueError('Known-empty diary fixture requires an offline disposable character')
    versions = [row[0] for row in db.execute('SELECT version FROM _rust_migrations ORDER BY version')]
    if len(versions) not in (8, 9, 10, 11) or versions != list(range(1, len(versions) + 1)):
        raise ValueError('Unsupported owned fixture schema')
    if not db.in_transaction:
        db.execute('BEGIN')
    # Only the new clone is passed by the harness. Use the server migration text
    # and the caller's transaction; executescript would silently commit early.
    for version, name in ((9, 'tower_arrival'), (10, 'tower_diary'), (11, 'tower_bones_offered')):
        if version <= len(versions):
            continue
        sql = (ROOT / f'crates/storage/migrations/V{version}__{name}.sql').read_text()
        import sqlite3
        statement = ''
        for line in sql.splitlines(keepends=True):
            statement += line
            if sqlite3.complete_statement(statement):
                db.execute(statement)
                statement = ''
        if statement.strip():
            raise ValueError('Incomplete fixture migration statement')
        db.execute('INSERT INTO _rust_migrations VALUES(?)', (version,))
    # INSERT deliberately fails if the fixture already has authoritative marks.
    db.execute('INSERT INTO tower_diary VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)',
               [row[0]] + [0] * 10 + [1, 0])
    db.execute('''INSERT INTO tower_arrival(character_id,bones_offered) VALUES(?,0)
        ON CONFLICT(character_id) DO UPDATE SET bones_offered=0''', (row[0],))


def prepare_kotlin(directory):
    # An existing character directory is never cleared or overwritten.
    directory.mkdir(parents=True, exist_ok=False)
    (directory / 'schema').write_text('11\n', encoding='utf-8')
    (directory / '.migrated').write_text('1\n', encoding='utf-8')
