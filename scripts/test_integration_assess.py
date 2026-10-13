import importlib.util,json,tempfile,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('assessor',Path(__file__).with_name('integration-assess.py'))
assessor=importlib.util.module_from_spec(spec);spec.loader.exec_module(assessor)

class AssessmentTests(unittest.TestCase):
    def run_case(self,log,mode='actions',exitcode=0,forced=False):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            (root/'client.log').write_text(log)
            (root/'result.json').write_text(json.dumps({'mode':mode,'clientExit':exitcode}))
            (root/'shutdown.json').write_text(json.dumps({'exitCode':0,'forced':forced,'exited':True}))
            (root/'logout-observation.json').write_text(json.dumps({'save_completed':True}))
            return assessor.assess(root)
    def test_zero_exit_without_effects_is_failure(self):
        self.assertEqual(self.run_case('Live native login accepted')['status'],'FAIL')
    def test_targeted_success_requires_all_markers(self):
        self.assertEqual(self.run_case('\n'.join(assessor.RULES['actions']))['status'],'PASS')
    def test_forced_shutdown_is_not_pass(self):
        self.assertEqual(self.run_case('\n'.join(assessor.RULES['actions']),forced=True)['status'],'FAIL')
    def test_continue_opcode_mention_is_not_success(self):
        self.assertEqual(self.run_case('Live native login accepted\nNative Continue click; no packet 115','dialogue-ui')['status'],'FAIL')
    def test_preauthentication_timeout_is_not_a_continue_failure(self):
        self.assertEqual(self.run_case('Error: no NPC response confirmed within bounded scenario','dialogue-ui',exitcode=1)['status'],'BLOCKED')
    def test_continue_requires_opcode_from_the_successful_click(self):
        prefix='Live native login accepted\nServer NPC response: Hello\nScenario passed; protocol_probe=false\n'
        for click in ('handled=false, outbound=[115]', 'handled=true, outbound=[1115]', 'handled=true, outbound=[]\nUnrelated packet 115'):
            self.assertEqual(self.run_case(prefix+'Native Continue click: '+click,'dialogue-ui')['status'],'FAIL')
        self.assertEqual(self.run_case(prefix+'Native Continue click: handled=true, outbound=[89, 115]','dialogue-ui')['status'],'PASS')
    def test_missing_lifecycle_is_blocked(self):
        with tempfile.TemporaryDirectory() as tmp:self.assertEqual(assessor.assess(Path(tmp))['status'],'BLOCKED')
    def test_missing_target_does_not_claim_continue_rejection(self):
        result=self.run_case('Live native login accepted\nError: no NPC response confirmed within bounded scenario','dialogue-ui',exitcode=1)
        self.assertEqual(result['status'],'FAIL')
        self.assertEqual(result['reason'],'NPC target selection not observed; Continue UNKNOWN')
        result=self.run_case('Live native login accepted\nTalk-to man definition 3107','dialogue-ui',exitcode=1)
        self.assertEqual(result['reason'],'NPC greeting not observed; Continue UNKNOWN')
    def test_logged_script_failure_overrides_zero_exit(self):
        self.assertEqual(self.run_case('\n'.join(assessor.RULES['login'])+'\nFailed Interface tick: example','login')['status'],'FAIL')
    def test_inventory_without_authentication_is_not_a_server_item_failure(self):
        result=self.run_case('Error: inventory scenario timed out at stage 0','inventory',exitcode=1)
        self.assertEqual(result['status'],'BLOCKED')
        self.assertIn('UNKNOWN',result['reason'])
    def test_inventory_readiness_without_dispatch_leaves_operation_unknown(self):
        result=self.run_case('Live native login accepted\nError: inventory scenario timed out at stage 0','inventory',exitcode=1)
        self.assertEqual(result['status'],'FAIL')
        self.assertIn('dispatch not observed',result['reason'])
    def test_prayer_markers_require_exact_native_packet_sequence(self):
        log='\n'.join(assessor.RULES['prayer'])
        self.assertEqual(self.run_case(log,'prayer')['status'],'FAIL')
        dispatch='\n'.join(f'Native prayer operation: component={component}, outbound=[47]' for component in [(541<<16)|9,(541<<16)|9,(541<<16)|10])
        success=log+'\n'+dispatch+'\nNative prayer Continue: outbound=[115]'
        result=self.run_case(success,'prayer')
        self.assertEqual(result['status'],'PASS')
        self.assertIn('may be predicted',result['limitations'][0])
        self.assertEqual(self.run_case(success.replace('[115]','[1115]'),'prayer')['status'],'FAIL')
        self.assertEqual(self.run_case(success.replace('[47]','[147]'),'prayer')['status'],'FAIL')
    def test_prayer_timeout_before_readiness_is_unknown(self):
        result=self.run_case('Error: prayer scenario deadline at stage 0','prayer',exitcode=1)
        self.assertEqual(result['status'],'BLOCKED')
        self.assertIn('UNKNOWN',result['reason'])

if __name__=='__main__':unittest.main()
