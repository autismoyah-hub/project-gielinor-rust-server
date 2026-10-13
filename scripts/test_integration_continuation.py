import json
import tempfile
import unittest
from pathlib import Path
from integration_continuation import continuation


class ContinuationTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

    def make(self, environment, **changes):
        run = self.root / environment
        run.mkdir(exist_ok=True)
        files = {
            'preparation.json': dict(environment=environment, fixture='canonical-basic', baseline='client-pin'),
            'result.json': dict(clientExit=0, clientHash='same-driver', mode='inventory'),
            'scenario-result.json': dict(status='PASS'),
            'logout-observation.json': dict(save_completed=True),
            'shutdown.json': dict(forced=False, exitCode=0),
        }
        files.update(changes)
        for name, value in files.items():
            (run / name).write_text(json.dumps(value))
        return run

    def test_equivalent_lineage_matches_across_environment_and_run_names(self):
        self.assertEqual(continuation(self.make('kotlin'), 'kotlin'), continuation(self.make('rust'), 'rust'))

    def test_different_input_procedure_does_not_match(self):
        a = continuation(self.make('kotlin'), 'kotlin')
        for field in ('mode', 'clientHash'):
            result = dict(clientExit=0, clientHash='same-driver', mode='inventory')
            result[field] = 'different'
            b = continuation(self.make('rust', **{'result.json': result}), 'rust')
            self.assertNotEqual(a, b)

    def test_failed_or_unverified_predecessors_rejected(self):
        for filename, data in [
            ('scenario-result.json', dict(status='FAIL')),
            ('logout-observation.json', dict(save_completed=False)),
            ('shutdown.json', dict(forced=True, exitCode=0)),
        ]:
            with self.assertRaises(ValueError):
                continuation(self.make('rust', **{filename: data}), 'rust')
        with self.assertRaises(ValueError):
            continuation(self.make('rust'), 'kotlin')

    def test_same_lineage_does_not_hide_initial_state_divergence(self):
        from integration_telemetry import compare
        from test_integration_telemetry import TelemetryTests
        factory=TelemetryTests()
        a=factory.events(self.root,'a')
        b=factory.events(self.root,'b')
        a[0]['data']['fixture_id']=continuation(self.make('kotlin'),'kotlin')
        b[0]['data']['fixture_id']=continuation(self.make('rust'),'rust')
        b[1]['data']['coins']=999
        report=compare(a,b)
        self.assertEqual(report['status'],'FAIL')
        self.assertEqual(report['first_divergence']['subsystem'],'state.initial')


if __name__ == '__main__':
    unittest.main()
