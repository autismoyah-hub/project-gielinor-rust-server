"""Identify matched scenario lineage without hiding persisted-state differences."""
import argparse
import hashlib
import json
from pathlib import Path


def continuation(run, environment):
    run = Path(run)
    read = lambda name: json.loads((run / name).read_text(encoding='utf-8-sig'))
    preparation = read('preparation.json')
    result = read('result.json')
    verdict = read('scenario-result.json')
    logout = read('logout-observation.json')
    shutdown = read('shutdown.json')
    if preparation.get('environment') != environment:
        raise ValueError('Predecessor environment mismatch')
    if verdict.get('status') != 'PASS' or result.get('clientExit') != 0:
        raise ValueError('Cannot continue a failed or incomplete scenario')
    if logout.get('save_completed') is not True or shutdown.get('forced') is not False or shutdown.get('exitCode') != 0:
        raise ValueError('Predecessor save or clean shutdown is unverified')
    lineage = {
        'fixture': preparation.get('fixture'),
        'scenario': result.get('mode'),
        'client': result.get('clientHash'),
        'baseline': preparation.get('baseline'),
    }
    if not all(isinstance(value, str) and value for value in lineage.values()):
        raise ValueError('Incomplete predecessor provenance')
    # This identifies the input procedure, not an assertion that resulting game
    # states match. The telemetry comparator still compares every initial field.
    digest = hashlib.sha256(json.dumps(lineage, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    return 'continued-' + digest


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('run', type=Path)
    parser.add_argument('environment', choices=('kotlin', 'rust'))
    args = parser.parse_args()
    print(continuation(args.run, args.environment))
