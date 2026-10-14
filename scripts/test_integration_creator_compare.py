import json
import struct
import tempfile
import unittest
from pathlib import Path
from integration_creator_compare import compare
from integration_telemetry import Writer


class CreatorComparisonTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.roots = [Path(self.tmp.name) / name for name in ('kotlin', 'rust')]
        fixture = Path(__file__).resolve().parent.parent / 'crates/server/tests/fixtures/tutorial-bootstrap.expected'
        row = next(row for row in fixture.read_text().splitlines() if row.startswith('0|0|0|-1|'))
        self.block = [(int(op), bytes.fromhex(data)) for op, data in
                      (entry.split(':', 1) for entry in row.split('|')[6].split(','))]
        for root in self.roots:
            root.mkdir()
            (root / 'scenario-result.json').write_text(json.dumps(dict(run_id=root.name, scenario='login', status='PASS', reason='Fixture acceptance')))
            (root / 'preparation.json').write_text(json.dumps(dict(creatorResume=True, environment=root.name,
                                                                 mode='login', fixture='fixture', baseline='pin', serverHash='server')))
            maps = sorted({(x << 8) | z for x in range(25, 28) for z in range(94, 97)} | {12337, 12843, 12849, 12851})
            arguments = ['--spawn', '1686,6115,0'] if root.name == 'rust' else ['-Dgielinor.integration.maps=' + ','.join(map(str, maps))]
            (root / 'server.json').write_text(json.dumps(dict(binaryHash='server', arguments=arguments)))
            writer = Writer(root / 'events.jsonl', root.name, 'login', root.name, 'test')
            writer.emit('context', dict(input_mode='login', fixture_id='fixture', baseline_id='pin',
                                        client_sha256='client', cache_id='cache', revision='239.1', server_sha256='server'), [])
            writer.emit('state.initial', {'coords': [0, 1686, 6115], 'vars': {'1': 0}}, [])
            writer.emit('persistence.completion', {'save_completed': True}, [])
            writer.emit('session.end', dict(client_exit=0, server_exit=0, forced=False), [])
            writer.close()
            self.capture(root, self.block)

    def capture(self, root, block):
        packets = block + [(28, b'')]
        (root / 'packets.bin').write_bytes(b'R239CAP1' + b''.join(struct.pack('>HI', op, len(data)) + data for op, data in packets))

    def change_event(self, root, kind, field, value):
        path = root / 'events.jsonl'
        rows = [json.loads(row) for row in path.read_text().splitlines()]
        next(row for row in rows if row['kind'] == kind)['data'][field] = value
        path.write_text(''.join(json.dumps(row) + '\n' for row in rows))

    def test_identical_sequences_match_only_scoped_wire_coverage(self):
        result = compare(*self.roots)
        self.assertEqual(result['status'], 'PASS')
        self.assertTrue(result['limitations'])
        self.assertEqual(result['normalization'], [])

    def test_equal_bootstrap_with_different_player_info_order_fails(self):
        self.capture(self.roots[1], [(28, b'')] + self.block)
        report = compare(*self.roots)
        self.assertEqual(report['status'], 'FAIL')
        self.assertEqual(report['bootstrap_position'], ['before', 'after'])

    def test_native_failure_remains_explicit_when_wire_matches(self):
        path = self.roots[1] / 'scenario-result.json'
        path.write_text(json.dumps(dict(run_id='rust', scenario='login', status='FAIL',
                                        reason='client opcode unsupported',
                                        first_failing_action={'line': 5, 'category': 'client_script_or_packet_error'})))
        report = compare(*self.roots)
        self.assertEqual(report['status'], 'PASS')  # Scope is strictly the wire sequence.
        self.assertEqual([entry['status'] for entry in report['native_acceptance']], ['PASS', 'FAIL'])
        self.assertEqual(report['native_acceptance'][1]['first_failing_action']['line'], 5)
        self.assertTrue(any('cannot override' in value for value in report['limitations']))

    def test_missing_native_assessment_blocks_comparison(self):
        (self.roots[1] / 'scenario-result.json').unlink()
        self.assertEqual(compare(*self.roots)['status'], 'BLOCKED')

    def test_payload_difference_is_not_normalized(self):
        block = list(self.block)
        index = next(i for i, (op, _) in enumerate(block) if op == 63)
        op, payload = block[index]
        block[index] = (op, bytes([payload[0] ^ 1]) + payload[1:])
        self.capture(self.roots[1], block)
        report = compare(*self.roots)
        self.assertEqual(report['status'], 'FAIL')
        self.assertEqual(report['first_divergence']['ordinal'], index)

    def test_configuration_and_failed_lifecycle_block_equal_packets(self):
        self.change_event(self.roots[1], 'context', 'cache_id', 'other-cache')
        self.assertEqual(compare(*self.roots)['status'], 'BLOCKED')
        self.change_event(self.roots[1], 'context', 'cache_id', 'cache')
        self.change_event(self.roots[1], 'session.end', 'forced', True)
        self.assertEqual(compare(*self.roots)['status'], 'BLOCKED')

    def test_mismatched_initial_state_or_invalid_capture_is_blocked(self):
        self.change_event(self.roots[1], 'state.initial', 'coords', [0, 3200, 3200])
        self.assertEqual(compare(*self.roots)['status'], 'BLOCKED')
        self.change_event(self.roots[1], 'state.initial', 'coords', [0, 1686, 6115])
        (self.roots[1] / 'packets.bin').write_bytes(b'R239CAP1\0')
        self.assertIn('Truncated', compare(*self.roots)['reason'])

    def test_wrong_scene_or_server_build_is_not_comparable(self):
        path = self.roots[1] / 'server.json'
        server = json.loads(path.read_text())
        server['arguments'] = ['--spawn', '3222,3218,0']
        path.write_text(json.dumps(server))
        self.assertEqual(compare(*self.roots)['status'], 'BLOCKED')
        server['arguments'] = ['--spawn', '1686,6115,0']
        server['binaryHash'] = 'other-server'
        path.write_text(json.dumps(server))
        self.assertIn('build identities differ', compare(*self.roots)['reason'])


if __name__ == '__main__':
    unittest.main()
