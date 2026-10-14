"""Extract initial creator wire output; never equate this with rendered acceptance."""
import argparse
import hashlib
import json
import struct
from pathlib import Path


def extract(path):
    packets = []
    if path.stat().st_size > 64 * 1024 * 1024:
        raise ValueError('Capture size bound exceeded')
    with path.open('rb') as stream:
        if stream.read(8) != b'R239CAP1':
            raise ValueError('Unknown capture format')
        while header := stream.read(6):
            if len(header) != 6:
                raise ValueError('Truncated capture header')
            opcode, size = struct.unpack('>HI', header)
            if size > 65535 or len(packets) >= 100000:
                raise ValueError('Capture packet bound exceeded')
            payload = stream.read(size)
            if len(payload) != size:
                raise ValueError('Truncated capture payload')
            packets.append((opcode, payload))
    first_info = next((i for i, (op, _) in enumerate(packets) if op == 28), None)
    start = next((i for i, (op, payload) in enumerate(packets)
                  if op == 114 and payload == b'\0' + struct.pack('>I', 2644)), None)
    if start is None or first_info is None:
        raise ValueError('Tutorial initialization or first player info not observed')
    block = []
    guides = 0
    for op, payload in packets[start:]:
        if op == 83:
            break
        block.append((op, payload))
        if op == 114 and len(payload) >= 5 and payload[-4:] == struct.pack('>I', 1974):
            guides += 1
            if guides == 2:
                break
    opened = []
    for ordinal, (op, payload) in enumerate(block):
        if op == 7:
            if len(payload) != 7:
                raise ValueError('Malformed interface open')
            opened.append((ordinal, (payload[1] << 8) | ((payload[0] - 128) & 255), payload[6]))
    overlay = next((i for i, group, kind in opened if group == 614 and kind == 1), None)
    creator = next((i for i, group, kind in opened if group == 679 and kind == 0), None)
    if guides != 2 or overlay is None or creator is None or overlay >= creator:
        raise ValueError('Complete overlay/creator/guide bootstrap not observed')
    return {'bootstrap_packets': [{'opcode': op, 'payload_hex': payload.hex()} for op, payload in block],
            'capture_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
            'bootstrap_before_first_player_info': start + len(block) <= first_info,
            'bootstrap_position': ('before' if start + len(block) <= first_info else 'after' if start > first_info else 'straddles'),
            'bootstrap_start_ordinal': start, 'bootstrap_end_ordinal': start + len(block) - 1,
            'first_player_info_ordinal': first_info,
            'limitations': ['Wire output only; rendered visibility, hidden appearance decoding, creator button actions and new-account creation are not validated']}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('capture', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    result = extract(args.capture)
    with args.output.open('x', encoding='utf8') as out:
        json.dump(result, out, indent=2)
    print('Observed initial tutorial creator wire sequence; see explicit coverage limitations')


if __name__ == '__main__':
    main()
