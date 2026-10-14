import sqlite3
import unittest
from integration_tower_diary_fixture import prepare

class TowerDiaryFixture(unittest.TestCase):
    def setUp(self):
        self.db=sqlite3.connect(':memory:');self.addCleanup(self.db.close)
        self.db.executescript("""CREATE TABLE accounts(id INTEGER,login_username TEXT);
            CREATE TABLE characters(id INTEGER PRIMARY KEY,account_id INTEGER,realm_id INTEGER,world_id INTEGER,x INTEGER,z INTEGER,level INTEGER);
            CREATE TABLE inventories(id INTEGER PRIMARY KEY,character_id INTEGER,inv_type INTEGER,UNIQUE(character_id,inv_type));
            CREATE TABLE inventory_objs(inventories_id INTEGER,slot INTEGER,obj INTEGER,count INTEGER,vars INTEGER);
            INSERT INTO accounts VALUES(1,'integration239'),(2,'unrelated');
            INSERT INTO characters VALUES(1,1,1,NULL,3224,3218,0),(2,2,1,NULL,3200,3200,0);
            INSERT INTO inventories VALUES(1,1,93),(2,1,94),(3,1,95),(4,2,93);
            INSERT INTO inventory_objs VALUES(1,0,995,1000,0),(2,3,1277,1,0),(3,0,1741,5,0),(4,0,995,777,0);""")
    def test_new_fixture_has_exact_five_hides_without_touching_other_character(self):
        with self.db:prepare(self.db)
        self.assertEqual(self.db.execute('SELECT x,z,level FROM characters WHERE id=1').fetchone(),(10424,120,0))
        self.assertEqual(self.db.execute('SELECT inventories_id,slot,obj,count,vars FROM inventory_objs WHERE inventories_id<4 ORDER BY slot').fetchall(),
                         [(1,i,1739,1,0) for i in range(5)])
        self.assertEqual(self.db.execute('SELECT count FROM inventory_objs WHERE inventories_id=4').fetchone(),(777,))
    def test_online_source_is_rejected_without_any_changes(self):
        self.db.execute('UPDATE characters SET world_id=1 WHERE id=1')
        before=self.db.execute('SELECT * FROM inventory_objs').fetchall()
        with self.assertRaises(ValueError):prepare(self.db)
        self.assertEqual(self.db.execute('SELECT * FROM inventory_objs').fetchall(),before)
if __name__=='__main__':unittest.main()
