from contextlib import closing
import sqlite3
import tempfile
import unittest
from pathlib import Path
from integration_diary_state import kotlin, rust


class DiaryEvidence(unittest.TestCase):
    def test_kotlin_integer_limits_caps_and_existence_flags(self):
        with tempfile.TemporaryDirectory(dir=Path(__file__).resolve().parent.parent / 'work/tmp') as temp:
            root = Path(temp)
            for name, value in {'tans': '2147483648', 'offers': '-9', 'bows': '  +99\n',
                                'shrimp-cooked': '1_0', 'minds-made': '5', 'arrows-made': '15',
                                'schema': '12', 'bounty': '99', 'mind-cast': 'false',
                                'needle': '', 'pending-11': '0'}.items():
                (root / name).write_text(value)
            state = kotlin(root)
            self.assertEqual(state['counts'], [0, 0, 3, 0, 5, 15])
            self.assertEqual(state['bounty'], 11)
            self.assertTrue(state['schema11'])
            self.assertTrue(state['actions'][2])
            self.assertTrue(state['done'][0])
            self.assertEqual(state['pending'], [False, True])

    def test_absent_rust_is_unknown_and_known_empty_matches_kotlin(self):
        with tempfile.TemporaryDirectory(dir=Path(__file__).resolve().parent.parent / 'work/tmp') as temp:
            root = Path(temp)
            path = root / 'state.sqlite'
            with closing(sqlite3.connect(path)) as db, db:
                db.executescript("""CREATE TABLE accounts(id INTEGER,login_username TEXT);
                    CREATE TABLE characters(id INTEGER,account_id INTEGER,realm_id INTEGER);
                    INSERT INTO accounts VALUES(1,'integration239');
                    INSERT INTO characters VALUES(1,1,1);""")
            self.assertEqual(rust(path), {'known': False})
            with closing(sqlite3.connect(path)) as db, db:
                db.executescript((Path(__file__).resolve().parent.parent / 'crates/storage/migrations/V10__tower_diary.sql').read_text())
            self.assertEqual(rust(path), {'known': False})
            with closing(sqlite3.connect(path)) as db, db:
                db.execute('INSERT INTO tower_diary VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)', [1] + [0] * 12)
            expected = kotlin(root / 'absent-canonical-character')
            self.assertIsNone(rust(path)['bones_offered'])
            expected['bones_offered'] = None
            self.assertEqual(rust(path), expected)
            with closing(sqlite3.connect(path)) as db, db:
                db.execute('CREATE TABLE tower_arrival(character_id INTEGER,bones_offered INTEGER)')
                db.execute('INSERT INTO tower_arrival VALUES(1,0)')
            self.assertEqual(rust(path), kotlin(root / 'absent-canonical-character'))
            with closing(sqlite3.connect(path)) as db, db:
                db.execute('UPDATE tower_arrival SET bones_offered=1')
            self.assertTrue(rust(path)['bones_offered'])
            with closing(sqlite3.connect(path)) as db, db:
                db.execute('UPDATE tower_arrival SET bones_offered=2')
            with self.assertRaises(ValueError): rust(path)
            with closing(sqlite3.connect(path)) as db, db:
                db.execute('UPDATE tower_arrival SET bones_offered=NULL')
            with closing(sqlite3.connect(path)) as db, db:
                db.execute('UPDATE tower_diary SET actions_mask=85,done_mask=1025,pending_mask=2')
            state = rust(path)
            self.assertEqual(state['actions'], [True, False, True, False, True, False, True])
            self.assertEqual([i for i, done in enumerate(state['done']) if done], [0, 10])
            self.assertEqual(state['pending'], [False, True])


if __name__ == '__main__':
    unittest.main()
