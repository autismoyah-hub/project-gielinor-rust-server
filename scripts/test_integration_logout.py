import unittest
from integration_logout import completed

class LogoutObservationTests(unittest.TestCase):
    def test_old_offline_fixture_is_not_a_completed_run(self):
        old={'world_id':None,'last_logout':'old'}
        self.assertFalse(completed(old,old))
    def test_new_logout_while_online_is_not_complete(self):
        self.assertFalse(completed({'last_logout':'old'},{'world_id':1,'last_logout':'new'}))
    def test_new_offline_save_is_complete(self):
        self.assertTrue(completed({'last_logout':'old'},{'world_id':None,'last_logout':'new'}))
    def test_missing_logout_is_unknown(self):
        self.assertFalse(completed({'last_logout':None},{'world_id':None,'last_logout':None}))
if __name__=='__main__':unittest.main()
