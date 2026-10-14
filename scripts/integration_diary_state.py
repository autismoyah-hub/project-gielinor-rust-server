"""Read-only diary persistence evidence, separate from arrival-state projection.

Kotlin values use restoreDiaryCounts' decoding before syncDiary/gameplay side
effects. The caller must supply the canonical post-migration character directory.
An absent Rust row is UNKNOWN, never equivalent to a known empty Kotlin diary.
"""
import argparse
import json
import re
import sqlite3
from contextlib import closing
from pathlib import Path

COUNTS = ('tans', 'offers', 'bows', 'shrimp-cooked', 'minds-made', 'arrows-made')
CAPS = (5, 5, 3, 5, 5, 15)
ACTIONS = ('vambrace-crafted', 'scimitar-smithed', 'mind-cast', 'guam', 'newt', 'potion-mixed', 'arrow-hit')
DONE = ('needle', 'offered', 'vambraces', 'scimitar', 'grove', 'shrimp', 'minds', 'potion', 'arrows', 'lite', 'cleared')
PENDING = ('pending-10', 'pending-11')


def kotlin(directory):
    def count(name, cap):
        path = directory / name
        if not path.exists():
            return 0
        raw = path.read_text(encoding='utf-8').strip()
        # Kotlin toIntOrNull rejects separators and values outside signed Int.
        if not re.fullmatch(r'[+-]?\d+', raw):
            return 0
        try:
            value = int(raw)
        except ValueError:
            return 0
        return max(0, min(value, cap)) if -2147483648 <= value <= 2147483647 else 0
    return {'known': True, 'counts': [count(name, cap) for name, cap in zip(COUNTS, CAPS)],
            'actions': [(directory / name).exists() for name in ACTIONS],
            'done': [(directory / name).exists() for name in DONE],
            'pending': [(directory / name).exists() for name in PENDING],
            'bounty': count('bounty', 11), 'schema11': count('schema', 11) == 11,
            'imp_bones': (directory / 'imp-bones').exists(),
            'bones_offered': (directory / 'bones').exists()}


def rust(database):
    with closing(sqlite3.connect(database.resolve().as_uri() + '?mode=ro', uri=True)) as db:
        row = db.execute("""SELECT c.id FROM characters c JOIN accounts a ON a.id=c.account_id
            WHERE a.login_username='integration239' AND c.realm_id=1""").fetchone()
        if row is None:
            raise ValueError('Disposable character missing')
        character = row[0]
        if not db.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='tower_diary'").fetchone():
            return {'known': False}
        row = db.execute('''SELECT tans,offers,bows,shrimp_cooked,minds_made,arrows_made,
            actions_mask,done_mask,pending_mask,bounty,schema11,imp_bones
            FROM tower_diary WHERE character_id=?''', (character,)).fetchone()
        if row is None:
            return {'known': False}
        if any(type(value) is not int or not -2147483648 <= value <= 2147483647 for value in row):
            raise ValueError('Invalid persisted diary integer')
        if not (0 <= row[6] < 128 and 0 <= row[7] < 2048 and 0 <= row[8] < 4
                and row[10] in (0, 1) and row[11] in (0, 1)):
            raise ValueError('Invalid persisted diary mask')
        bones_offered = None
        if any(r[1] == 'bones_offered' for r in db.execute('PRAGMA table_info(tower_arrival)')):
            flag = db.execute('SELECT bones_offered FROM tower_arrival WHERE character_id=?', (character,)).fetchone()
            if flag and flag[0] is not None:
                if type(flag[0]) is not int or flag[0] not in (0, 1):
                    raise ValueError('Invalid bones-offered flag')
                bones_offered = bool(flag[0])
        return {'known': True, 'counts': list(row[:6]),
                'actions': [bool(row[6] & (1 << bit)) for bit in range(7)],
                'done': [bool(row[7] & (1 << bit)) for bit in range(11)],
                'pending': [bool(row[8] & (1 << bit)) for bit in range(2)],
                'bounty': row[9], 'schema11': bool(row[10]), 'imp_bones': bool(row[11]),
                'bones_offered': bones_offered}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('environment', choices=('rust', 'kotlin'))
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    state = rust(args.source) if args.environment == 'rust' else kotlin(args.source)
    with args.output.open('x', encoding='utf-8') as output:
        json.dump({'stage': 'persisted-fields-before-session-sync', 'state': state}, output, indent=2)


if __name__ == '__main__':
    main()
