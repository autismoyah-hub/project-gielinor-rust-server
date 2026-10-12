"""Summarize native R239CAP1 captures without treating timing differences as parity.

Raw ordered hashes remain available. Selected stable state fields are decoded;
NPC movement and session-specific player indices are intentionally not equated.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct


def read_capture(path):
    data = path.read_bytes()
    if data[:8] != b"R239CAP1":
        raise ValueError("not a native client packet capture")
    offset, records = 8, []
    while offset < len(data):
        if offset + 6 > len(data):
            raise ValueError("truncated packet header")
        opcode, size = struct.unpack_from(">HI", data, offset)
        offset += 6
        if offset + size > len(data):
            raise ValueError("truncated packet payload")
        records.append((opcode, data[offset:offset + size]))
        offset += size
    return data, records


def summary(path, names):
    data, records = read_capture(path)
    counts = Counter(op for op, _ in records)
    result = {
        "capture": str(path), "sha256": hashlib.sha256(data).hexdigest(),
        "packet_count": len(records),
        "counts": {names.get(op, str(op)): n for op, n in sorted(counts.items())},
        "first_96_opcodes": [names.get(op, str(op)) for op, _ in records[:96]],
        "state": {"initial_position": None, "stats": {}, "run_energy": None},
        "first_payloads": {},
    }
    for op, payload in records:
        name = names.get(op, str(op))
        result["first_payloads"].setdefault(name, {
            "length": len(payload), "sha256": hashlib.sha256(payload).hexdigest(),
            "hex": payload.hex() if len(payload) <= 128 else None,
        })
        if op == 49 and result["state"]["initial_position"] is None:
            if len(payload) != 4614:
                raise ValueError("unexpected initial rebuild size")
            packed = int.from_bytes(payload[:4], "big") >> 2
            result["state"]["initial_position"] = [packed >> 28, (packed >> 14) & 16383, packed & 16383]
        elif op == 46:
            if len(payload) != 7:
                raise ValueError("unexpected stat size")
            invisible, current, index, a, b, c, d = payload
            xp = int.from_bytes(bytes([c, d, a, b]), "big", signed=True)
            result["state"]["stats"][str((index - 128) & 255)] = {"current": current, "invisible": invisible, "xp": xp}
        elif op == 64:
            result["state"]["run_energy"] = int.from_bytes(payload, "big")
    return result


def main():
    p = argparse.ArgumentParser()
    p.add_argument("capture", type=Path)
    p.add_argument("output", type=Path)
    p.add_argument("--compare", type=Path)
    args = p.parse_args()
    root = Path(__file__).resolve().parent.parent
    names = {}
    for line in (root / "reference/protocol/server.txt").read_text().splitlines():
        name, opcode, _ = line.split()
        names[int(opcode)] = name
    result = summary(args.capture, names)
    if args.compare:
        other = summary(args.compare, names)
        result["comparison"] = {
            "other": str(args.compare), "state_equal": result["state"] == other["state"],
            "state_other": other["state"],
            "only_this_packet_types": sorted(result["counts"].keys() - other["counts"].keys()),
            "only_other_packet_types": sorted(other["counts"].keys() - result["counts"].keys()),
            "different_first_payloads": sorted(name for name in result["first_payloads"].keys() & other["first_payloads"].keys() if result["first_payloads"][name] != other["first_payloads"][name]),
            "interpretation": "Differences require source/configuration/timing attribution; this is not a full gameplay parity verdict.",
        }
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(f"Summarized {result['packet_count']} native packets; initial position {result['state']['initial_position']}")


if __name__ == "__main__":
    main()
