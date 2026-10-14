"""Strict creator-resume wire comparison; never promotes wire parity to gameplay parity."""
import argparse
import hashlib
import json
from pathlib import Path
from integration_creator_capture import extract
from integration_telemetry import load


def compare(kotlin, rust):
    report = {'status': 'BLOCKED', 'scope': 'initial creator-resume wire sequence',
              'limitations': ['Rendered visibility, hidden-appearance decoding, button actions, new-account registration and later gameplay are UNKNOWN'],
              'normalization': [], 'runs': [str(kotlin), str(rust)], 'native_acceptance': []}
    contexts, initials, captures = [], [], []
    try:
        for root, environment in [(kotlin, 'kotlin'), (rust, 'rust')]:
            preparation = json.loads((root / 'preparation.json').read_text(encoding='utf-8-sig'))
            if preparation.get('creatorResume') is not True or preparation.get('environment') != environment or preparation.get('mode') != 'login':
                raise ValueError('Expected matched creator-resume login runs in Kotlin/Rust order')
            acceptance_path = root / 'scenario-result.json'
            acceptance = json.loads(acceptance_path.read_text(encoding='utf-8-sig'))
            if (acceptance.get('run_id') != root.name or acceptance.get('scenario') != 'login'
                    or acceptance.get('status') not in ('PASS', 'FAIL', 'BLOCKED')):
                raise ValueError('Missing or invalid native acceptance assessment')
            report['native_acceptance'].append({
                'environment': environment, 'status': acceptance['status'],
                'reason': acceptance.get('reason'),
                'first_failing_action': acceptance.get('first_failing_action'),
                'assessment_sha256': hashlib.sha256(acceptance_path.read_bytes()).hexdigest(),
            })
            if acceptance['status'] != 'PASS':
                report['limitations'].append(
                    f"{environment} native acceptance is {acceptance['status']}; equal wire output cannot override that outcome")
            rows = load(root / 'events.jsonl')

            def one(kind):
                found = [row for row in rows if row['kind'] == kind]
                if len(found) != 1 or found[0]['observation'] != 'observed' or found[0]['environment'] != environment:
                    raise ValueError(f'Missing or ambiguous observed {kind}')
                return found[0]['data']

            context = one('context')
            if context.get('input_mode') != 'login':
                raise ValueError('Capture input mode is not login')
            if context.get('fixture_id') != preparation.get('fixture') or context.get('baseline_id') != preparation.get('baseline'):
                raise ValueError('Preparation and telemetry identities differ')
            server = json.loads((root / 'server.json').read_text(encoding='utf-8-sig'))
            arguments = server.get('arguments', [])
            if environment == 'rust':
                binary = server.get('binaryHash')
                if not binary or binary != preparation.get('serverHash') or binary != context.get('server_sha256'):
                    raise ValueError('Rust server build identities differ')
                if '--spawn' not in arguments or arguments[arguments.index('--spawn') + 1:arguments.index('--spawn') + 2] != ['1686,6115,0']:
                    raise ValueError('Rust creator scene window was not configured')
            else:
                map_args = [arg for arg in arguments if arg.startswith('-Dgielinor.integration.maps=')]
                expected_maps = {(x << 8) | z for x in range(25, 28) for z in range(94, 97)} | {12337, 12843, 12849, 12851}
                if len(map_args) != 1 or {int(value) for value in map_args[0].split('=', 1)[1].split(',')} != expected_maps:
                    raise ValueError('Kotlin creator scene window was not configured')
            if one('persistence.completion').get('save_completed') is not True:
                raise ValueError('Logout save completion is unknown')
            end = one('session.end')
            if end.get('client_exit') != 0 or end.get('server_exit') != 0 or end.get('forced') is not False:
                raise ValueError('Failed or incomplete lifecycle')
            contexts.append(context)
            initials.append(one('state.initial'))
            captures.append(extract(root / 'packets.bin'))
        for key in ('baseline_id', 'client_sha256', 'fixture_id', 'cache_id', 'revision'):
            if not contexts[0].get(key) or contexts[0].get(key) != contexts[1].get(key):
                raise ValueError(f'Configuration/provenance mismatch: {key}')
        if initials[0] != initials[1]:
            raise ValueError('Initial character states differ')
    except (ValueError, OSError, KeyError) as error:
        return dict(report, reason=str(error))
    report['context'] = contexts
    report['capture_sha256'] = [capture['capture_sha256'] for capture in captures]
    report['telemetry_sha256'] = [hashlib.sha256((root / 'events.jsonl').read_bytes()).hexdigest() for root in (kotlin, rust)]
    report['server_record_sha256'] = [hashlib.sha256((root / 'server.json').read_bytes()).hexdigest() for root in (kotlin, rust)]
    report['bootstrap_position'] = [capture['bootstrap_position'] for capture in captures]
    if report['bootstrap_position'][0] != report['bootstrap_position'][1]:
        return dict(report, status='FAIL', reason='Bootstrap ordering relative to first player info differs')
    left, right = (capture['bootstrap_packets'] for capture in captures)
    report['packet_counts'] = [len(left), len(right)]
    for ordinal in range(max(len(left), len(right))):
        expected = left[ordinal] if ordinal < len(left) else None
        actual = right[ordinal] if ordinal < len(right) else None
        if expected != actual:
            return dict(report, status='FAIL', reason='Exact bootstrap packet sequence differs',
                        first_divergence={'ordinal': ordinal, 'kotlin': expected, 'rust': actual})
    return dict(report, status='PASS', reason='Exact initial creator wire sequence matches; coverage is limited to this sequence')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('kotlin', type=Path)
    parser.add_argument('rust', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    report = compare(args.kotlin, args.rust)
    with args.output.open('x', encoding='utf8') as stream:
        json.dump(report, stream, indent=2)
    print(report['status'] + ': ' + report['reason'])


if __name__ == '__main__':
    main()
