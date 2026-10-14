import struct
import tempfile
import unittest
from pathlib import Path
from integration_creator_capture import extract


class CreatorCaptureTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.path = Path(self.tmp.name) / 'packets.bin'
        source = Path(__file__).resolve().parent.parent / 'crates/server/tests/fixtures/tutorial-bootstrap.expected'
        row = next(row for row in source.read_text().splitlines() if row.startswith('0|0|0|-1|'))
        self.block = [(int(op), bytes.fromhex(data)) for op, data in
                      (entry.split(':', 1) for entry in row.split('|')[6].split(','))]

    def write(self, packets):
        self.path.write_bytes(b'R239CAP1' + b''.join(struct.pack('>HI', op, len(data)) + data for op, data in packets))

    def test_original_kotlin_bootstrap_is_extracted_without_later_world_packets(self):
        self.write([(49, b'')] + self.block + [(114, b'later'), (28, b'')])
        result = extract(self.path)
        self.assertEqual(result['bootstrap_packets'], [{'opcode': op, 'payload_hex': data.hex()} for op, data in self.block])

    def test_initial_appearance_before_tutorial_reports_observed_order(self):
        self.write([(49, b''), (28, b'')] + self.block)
        result = extract(self.path)
        self.assertFalse(result['bootstrap_before_first_player_info'])
        self.assertEqual(result['bootstrap_position'], 'after')
        self.assertEqual(result['first_player_info_ordinal'], 1)

    def test_tick_boundary_cannot_be_crossed_to_complete_bootstrap(self):
        self.write([(28, b'')] + self.block[:-1] + [(83, b'')] + self.block[-1:])
        with self.assertRaisesRegex(ValueError, 'Complete overlay/creator/guide'):
            extract(self.path)

    def test_player_info_inside_bootstrap_remains_in_exact_capture(self):
        self.write(self.block[:1] + [(28, b'')] + self.block[1:])
        result = extract(self.path)
        self.assertEqual(result['bootstrap_position'], 'straddles')
        self.assertEqual(result['bootstrap_packets'][1], {'opcode': 28, 'payload_hex': ''})

    def test_missing_modal_and_truncation_are_not_success(self):
        self.write([(op, data) for op, data in self.block if op != 7] + [(28, b'')])
        with self.assertRaises(ValueError):
            extract(self.path)
        self.path.write_bytes(b'R239CAP1\0')
        with self.assertRaises(ValueError):
            extract(self.path)


if __name__ == '__main__':
    unittest.main()
