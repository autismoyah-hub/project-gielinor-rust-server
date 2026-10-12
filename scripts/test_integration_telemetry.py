import copy,json,tempfile,unittest
from pathlib import Path
from integration_telemetry import Writer,load,compare,import_run
class TelemetryTests(unittest.TestCase):
    def events(self,root,name='a',amount=1000,exitcode=0):
        path=root/(name+'.jsonl');w=Writer(path,name,'bank','kotlin' if name=='a' else 'rust','test')
        w.emit('context',dict(baseline_id='pin',client_sha256='abc',fixture_id='fixture',cache_id='cache',input_mode='native',revision='239.1'),{})
        w.emit('state.initial',{'coins':1000},{})
        w.emit('action.effect',{'action':'bank.withdraw','coins':amount},{},game_tick=42 if name=='a' else 93)
        w.emit('state.persisted',{'coins':amount},{})
        w.emit('session.end',{'client_exit':exitcode,'server_exit':0,'forced':False,'server_exited':True},{})
        w.close();return load(path)
    def test_ids_and_absolute_clocks_do_not_create_differences(self):
        with tempfile.TemporaryDirectory() as t:
            a=self.events(Path(t));b=self.events(Path(t),'b');self.assertEqual(compare(a,b)['status'],'PASS')
    def test_first_gameplay_difference_is_preserved(self):
        with tempfile.TemporaryDirectory() as t:
            r=compare(self.events(Path(t)),self.events(Path(t),'b',999));self.assertEqual(r['status'],'FAIL');self.assertEqual(r['first_divergence']['subsystem'],'action.effect')
    def test_configuration_mismatch_is_not_a_server_defect(self):
        with tempfile.TemporaryDirectory() as t:
            a=self.events(Path(t));b=self.events(Path(t),'b');b[0]['data']['fixture_id']='other';self.assertEqual(compare(a,b)['status'],'BLOCKED')
    def test_equal_failures_are_never_parity(self):
        with tempfile.TemporaryDirectory() as t:self.assertEqual(compare(self.events(Path(t),exitcode=1),self.events(Path(t),'b',exitcode=1))['status'],'BLOCKED')
    def test_missing_coverage_is_not_success(self):
        with tempfile.TemporaryDirectory() as t:
            a=self.events(Path(t));b=self.events(Path(t),'b');self.assertEqual(compare([e for e in a if e['kind']!='state.initial'],[e for e in b if e['kind']!='state.initial'])['status'],'BLOCKED')
    def test_secret_and_oversize_records_rejected(self):
        with tempfile.TemporaryDirectory() as t:
            w=Writer(Path(t)/'x','x','x','rust','test')
            try:
                with self.assertRaises(ValueError):w.emit('action',{'password':'not-real'}, {})
                with self.assertRaises(ValueError):w.emit('action',{'text':'x'*20000}, {})
            finally:w.close()
    def test_append_only_and_duplicate_id_validation(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t);self.events(root)
            with self.assertRaises(FileExistsError):Writer(root/'a.jsonl','a','x','rust','test')
            with (root/'a.jsonl').open('a') as f:f.write((root/'a.jsonl').read_text().splitlines()[0]+'\n')
            with self.assertRaises(ValueError):load(root/'a.jsonl')
    def test_truncated_capture_rejected(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t);(root/'packets.bin').write_bytes(b'R239CAP1\x00')
            with self.assertRaises(ValueError):import_run(root,root/'x.jsonl','rust','login','pin',None,None)
    def test_relative_action_timing_not_normalized_away(self):
        with tempfile.TemporaryDirectory() as t:
            a=self.events(Path(t));b=self.events(Path(t),'b');x=copy.deepcopy(a[2]);y=copy.deepcopy(b[2]);x['game_tick']=44;y['game_tick']=96;a.insert(3,x);b.insert(3,y)
            self.assertEqual(compare(a,b)['classification'],'relative action timing differs')
if __name__=='__main__':unittest.main()
