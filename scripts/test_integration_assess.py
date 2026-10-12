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
            return assessor.assess(root)
    def test_zero_exit_without_effects_is_failure(self):
        self.assertEqual(self.run_case('Live native login accepted')['status'],'FAIL')
    def test_targeted_success_requires_all_markers(self):
        self.assertEqual(self.run_case('\n'.join(assessor.RULES['actions']))['status'],'PASS')
    def test_forced_shutdown_is_not_pass(self):
        self.assertEqual(self.run_case('\n'.join(assessor.RULES['actions']),forced=True)['status'],'FAIL')
    def test_continue_opcode_mention_is_not_success(self):
        self.assertEqual(self.run_case('Native Continue click; no packet 115','dialogue-ui')['status'],'FAIL')
    def test_missing_lifecycle_is_blocked(self):
        with tempfile.TemporaryDirectory() as tmp:self.assertEqual(assessor.assess(Path(tmp))['status'],'BLOCKED')
    def test_logged_script_failure_overrides_zero_exit(self):
        self.assertEqual(self.run_case('\n'.join(assessor.RULES['login'])+'\nFailed Interface tick: example','login')['status'],'FAIL')

if __name__=='__main__':unittest.main()
