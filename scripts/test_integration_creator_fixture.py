import json
import sqlite3
import unittest
from integration_creator_fixture import prepare_creator_resume


class CreatorFixtureTest(unittest.TestCase):
    def setUp(self):
        self.db = sqlite3.connect(':memory:')
        self.addCleanup(self.db.close)
        self.db.executescript('''
            CREATE TABLE accounts(id INTEGER,login_username TEXT);
            CREATE TABLE characters(id INTEGER,account_id INTEGER,realm_id INTEGER,
                world_id INTEGER,varps TEXT,x INTEGER,z INTEGER,level INTEGER,last_login TEXT);
            INSERT INTO accounts VALUES(1,'integration239');
            INSERT INTO characters VALUES(1,1,1,NULL,'{"9":77,"10":53,"11":1}',3224,3218,0,'existing');
        ''')

    def test_resume_preserves_unrelated_state_and_existing_account_identity(self):
        prepare_creator_resume(self.db, 10, 11)
        values,x,z,level,last = self.db.execute('SELECT varps,x,z,level,last_login FROM characters').fetchone()
        self.assertEqual(json.loads(values), {'9':77,'10':0,'11':0})
        self.assertEqual((x,z,level,last), (1686,6115,0,'existing'))

    def test_online_account_is_never_changed(self):
        self.db.execute('UPDATE characters SET world_id=1')
        before = self.db.execute('SELECT * FROM characters').fetchall()
        with self.assertRaises(ValueError):
            prepare_creator_resume(self.db, 10, 11)
        self.assertEqual(self.db.execute('SELECT * FROM characters').fetchall(), before)

    def test_invalid_identities_are_rejected(self):
        with self.assertRaises(ValueError):
            prepare_creator_resume(self.db, 10, 10)


if __name__ == '__main__':
    unittest.main()
