import copy,json,tempfile,unittest
from pathlib import Path
from integration_telemetry import Writer,load,compare,import_run
class TelemetryTests(unittest.TestCase):
    def test_prayer_prediction_does_not_become_server_acceptance(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)
            (root/'client.log').write_text('Native prayer operation: component=35454985, outbound=[47]\nClient observed Thick Skin active: bit4104=1\nObserved server prayer requirement notice: You need a Prayer level of 4.\nNative prayer Continue: outbound=[115]\nPrayer scenario observed activation, deactivation, requirement notice and native Continue closure\n')
            import_run(root,root/'events.jsonl','rust','prayer','pin','fixture','cache')
            rows=load(root/'events.jsonl')
            prayer=[e for e in rows if e['kind'].startswith('prayer.')]
            self.assertEqual([e['kind'] for e in prayer],['prayer.dispatch','prayer.client_state','prayer.continue'])
            self.assertTrue(all(e['data']['server_acceptance'] is None for e in prayer))
            self.assertEqual(prayer[0]['data']['outgoing_opcodes'],[47])
            self.assertEqual([e['data']['action'] for e in rows if e['kind']=='action.effect'],['prayer.requirement_notice','prayer.notice_closed'])
    def events(self,root,name='a',amount=1000,exitcode=0):
        path=root/(name+'.jsonl');w=Writer(path,name,'bank','kotlin' if name=='a' else 'rust','test')
        w.emit('context',dict(baseline_id='pin',client_sha256='abc',fixture_id='fixture',cache_id='cache',input_mode='native',revision='239.1'),{})
        w.emit('state.initial',{'coins':1000},{})
        w.emit('action.effect',{'action':'bank.withdraw','coins':amount},{},game_tick=42 if name=='a' else 93)
        w.emit('state.persisted',{'coins':amount},{})
        w.emit('persistence.completion',{'save_completed':True},{})
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
    def test_missing_logout_observation_is_unknown(self):
        with tempfile.TemporaryDirectory() as t:
            a=self.events(Path(t));b=self.events(Path(t),'b')
            a=[e for e in a if e['kind']!='persistence.completion']
            self.assertEqual(compare(a,b)['classification'],'logout save completion is UNKNOWN or incomplete')
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
    def test_continue_dispatch_and_response_are_observed_separately(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)
            (root/'client.log').write_text('Scenario initial position [0, 3224, 3218]; Talk-to man definition 3108, slot 34\nNative Continue click at Some([215, 450]): handled=true, outbound=[115]\nServer NPC response: Hello.\n')
            import_run(root,root/'events.jsonl','rust','dialogue-ui','pin','fixture','cache')
            rows=load(root/'events.jsonl')
            dispatch=next(e for e in rows if e['kind']=='client.continue')
            self.assertEqual(dispatch['data'],{'handled':True,'outgoing_opcodes':[115],'server_acceptance':None})
            self.assertEqual(next(e for e in rows if e['kind']=='dialogue.response')['data']['text'],'Hello.')
            self.assertEqual(next(e for e in rows if e['kind']=='persistence.completion')['observation'],'unavailable')
    def test_arrival_records_destination_and_container_effects(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)
            (root/'client.log').write_text('Arrival position=[0, 10424, 120]\nServer confirmed first Tower arrival: [10424, 120]\nServer confirmed first-arrival inventory and equipment cleared\n')
            import_run(root,root/'events.jsonl','kotlin','arrival','pin','fixture','cache')
            rows=load(root/'events.jsonl')
            effects=[e['data'] for e in rows if e['kind']=='action.effect']
            self.assertEqual([e['action'] for e in effects],['tower.first_arrival','tower.first_kit_clear'])
            self.assertEqual(effects[0]['parameters']['position'],[0,10424,120])
            self.assertEqual(effects[1]['parameters']['containers'],[93,94])
    def test_restore_coverage_requires_complete_driver_assertion(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)
            (root/'client.log').write_text('Native login restored worn sword\nNative login restored worn sword and its appearance, 1000 coins and two lobsters; position [0, 3224, 3218]\n')
            import_run(root,root/'events.jsonl','rust','inventory-verify','pin','fixture','cache')
            effects=[e['data'] for e in load(root/'events.jsonl') if e['kind']=='action.effect']
            self.assertEqual(len(effects),1)
            self.assertEqual(effects[0]['action'],'inventory.restore')
            self.assertEqual(effects[0]['parameters']['equipment'],{'container':94,'slot':3,'item':1277})
            self.assertEqual(effects[0]['parameters']['appearance_equipment_3'],3325)
    def test_inventory_dispatch_records_packet_separately_from_acceptance(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)
            (root/'client.log').write_text('Native interface operation 3: component=9764864, child=0, outbound=[47]\nNative interface operation 1: component=25362450, child=-1\n')
            import_run(root,root/'events.jsonl','rust','inventory','pin','fixture','cache')
            dispatch=[e['data'] for e in load(root/'events.jsonl') if e['kind']=='client.dispatch']
            self.assertEqual(dispatch[0]['operation'],3)
            self.assertEqual(dispatch[0]['outgoing_opcodes'],[47])
            self.assertEqual(dispatch[0]['dispatch_result'],'packet_queued')
            self.assertIsNone(dispatch[0]['server_acceptance'])
            self.assertIsNone(dispatch[1]['outgoing_opcodes'])
            self.assertEqual(dispatch[1]['dispatch_result'],'attempt_logged')
if __name__=='__main__':unittest.main()
