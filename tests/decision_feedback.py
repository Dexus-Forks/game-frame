#!/usr/bin/env python3
"""Test per-user decision corrections and the exported training set against sample-game evidence."""
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / 'tools/llm/decision_feedback.py'


def run(*args):
    return subprocess.run([sys.executable, str(SCRIPT), *map(str, args)],
                          capture_output=True, text=True, timeout=15)


def main(sample_exe):
    sample = subprocess.run([sys.executable, str(ROOT / 'tools/pe/peinfo.py'), str(sample_exe)],
                            capture_output=True, text=True, check=True).stdout
    assert 'EXE, x64 (64-bit)' in sample, sample
    request = {
        'state': {'sample_game_pe_analysis': sample,
                  'runtime_evidence': 'No emulator measurement or parity result is available.'},
        'questions': {
            'next_step': {
                'type': 'choice',
                'instructions': 'What is the safest next step before porting this behavior?',
                'criteria': {'measure': 'Measure the original and compare with a focused test.',
                             'trace': 'Trace more of the static code first.',
                             'port': 'Port the behavior now.'},
            },
            'parity_proven': {
                'type': 'noul',
                'instructions': 'Does this state show measured runtime parity?',
            },
            'evidence_level': {
                'type': 'score',
                'instructions': 'How strong is the parity evidence?',
                'criteria': ['Static code only', 'Original measured', 'Parity test passed'],
            },
        },
    }
    with tempfile.TemporaryDirectory(prefix='gameframe-decision-feedback-') as folder:
        root = Path(folder)
        ledger = root / 'user-profile'
        request_file = root / 'request.json'
        request_file.write_text(json.dumps(request), encoding='utf-8')
        base = ['--data-dir', ledger]
        for question, answer in [('next_step', 'measure'), ('parity_proven', 'false'),
                                 ('evidence_level', '0'), ('next_step', 'trace')]:
            result = run('record', request_file, '--question', question, '--answer', answer,
                         '--model', 'mock-decision-model', *base)
            assert result.returncode == 0, result.stderr
        if os.name != 'nt':
            assert (ledger / 'feedback.jsonl').stat().st_mode & 0o777 == 0o600
            assert ledger.stat().st_mode & 0o777 == 0o700

        small = root / 'small-export'
        result = run('export', '--output', small, *base)
        assert result.returncode == 0, result.stderr
        manifest = json.loads((small / 'manifest.json').read_text())
        if os.name != 'nt':
            assert small.stat().st_mode & 0o777 == 0o700
        assert manifest['ledger_events'] == 4 and manifest['unique_prompts'] == 3, manifest
        assert manifest['superseded_corrections'] == 1 and manifest['valid_examples'] == 0, manifest
        assert not (small / 'valid.jsonl').exists()
        rows = [json.loads(line) for line in (small / 'records.jsonl').read_text().splitlines()]
        corrected = [row for row in rows if row['question'] == request['questions']['next_step']['instructions']]
        assert len(corrected) == 1 and corrected[0]['answer_key'] == 'trace', corrected
        assert corrected[0]['messages'][0]['role'] == 'system'
        assert corrected[0]['messages'][2]['content'] in ('A', 'B', 'C')
        assert manifest['training_note'].startswith('Export only.'), manifest

        for index in range(9):
            request['state'] = {'case': index, 'sample_game_pe_analysis': sample,
                                'runtime_evidence': 'No runtime measurement.'}
            request_file.write_text(json.dumps(request), encoding='utf-8')
            result = run('record', request_file, '--question', 'next_step', '--answer', 'measure', *base)
            assert result.returncode == 0, result.stderr

        full = root / 'full-export'
        result = run('export', '--output', full, '--valid-fraction', '0.2', *base)
        assert result.returncode == 0, result.stderr
        manifest = json.loads((full / 'manifest.json').read_text())
        assert manifest['unique_prompts'] == 12 and manifest['distinct_states'] == 10, manifest
        assert 0 < manifest['valid_examples'] < manifest['unique_prompts'], manifest
        train = [json.loads(line) for line in (full / 'train.jsonl').read_text().splitlines()]
        valid = [json.loads(line) for line in (full / 'valid.jsonl').read_text().splitlines()]
        train_states = {json.dumps(row['messages'][1]['content'], sort_keys=True) for row in train}
        valid_states = {json.dumps(row['messages'][1]['content'], sort_keys=True) for row in valid}
        assert not train_states & valid_states, 'a state appeared in both splits'

        invalid = run('record', request_file, '--question', 'next_step', '--answer', 'not-an-option', *base)
        assert invalid.returncode == 1 and 'answer must be one of' in invalid.stderr, invalid
        existing = run('export', '--output', full, *base)
        assert existing.returncode == 1 and 'already exists' in existing.stderr, existing
    print('Decision feedback: explicit corrections, user-local file, dedupe, chat export, grouped holdout')


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('usage: tests/decision_feedback.py SAMPLE_GAME_EXE')
    main(sys.argv[1])
