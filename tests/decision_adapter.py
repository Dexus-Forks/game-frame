#!/usr/bin/env python3
"""Exercise generic decision routing with sample-game evidence and a mock System One endpoint."""
import json
import os
import subprocess
import sys
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / 'tools/llm/decision.py'


def main(sample_exe):
    sample = subprocess.run([sys.executable, str(ROOT / 'tools/pe/peinfo.py'), str(sample_exe)],
                            capture_output=True, text=True, check=True).stdout
    assert 'EXE, x64 (64-bit)' in sample, sample
    request = {
        'state': {
            'sample_game_pe_analysis': sample,
            'runtime_evidence': 'No emulator measurement or parity result is available.',
        },
        'questions': {
            'next_step': {
                'type': 'choice',
                'instructions': 'What is the safest next step before porting this behavior?',
                'criteria': {
                    'measure': 'Measure the original and compare with a focused test.',
                    'trace': 'Trace more of the static code first.',
                    'port': 'Port the behavior now.',
                    'stop': 'Stop work on the behavior.',
                },
            },
            'parity_proven': {
                'type': 'noul',
                'instructions': 'Does this state show measured runtime parity?',
                'criteria': {'true': 'A comparison proves parity.',
                             'false': 'No comparison proves parity.'},
            },
            'evidence_level': {
                'type': 'score',
                'instructions': 'How strong is the evidence that the port matches the original?',
                'criteria': ['Static output only', 'Static trace checked',
                             'Original measured', 'Parity test passed'],
            },
        },
    }
    captured = []
    authorization = []

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_args):
            pass

        def do_POST(self):
            assert self.path == '/v1/systemone', self.path
            captured.append(json.loads(self.rfile.read(int(self.headers['Content-Length']))))
            authorization.append(self.headers.get('Authorization'))
            answer = {
                'model': 'mock-decision-model',
                'answers': {
                    'next_step': {'choice': 'measure', 'probabilities': {'measure': 0.8},
                                  'confidence': 0.7},
                    'parity_proven': {'noul': 0.01},
                    'evidence_level': {
                        'score': 0.2,
                        'legend': request['questions']['evidence_level']['criteria'],
                        'probabilities': [0.8, 0.1, 0.08, 0.02], 'confidence': 0.7,
                    },
                },
            }
            body = json.dumps(answer).encode()
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Content-Length', str(len(body)))
            self.end_headers()
            self.wfile.write(body)

    server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        endpoint = f'http://127.0.0.1:{server.server_port}/v1/systemone'
        with tempfile.TemporaryDirectory(prefix='gameframe-decision-') as folder:
            source = Path(folder) / 'request.json'
            source.write_text(json.dumps(request), encoding='utf-8')
            result = subprocess.run(
                [sys.executable, str(SCRIPT), str(source), '--model', 'mock-decision-model',
                 '--url', endpoint, '--timeout', '5'], capture_output=True, text=True, timeout=10)
            assert result.returncode == 0, result.stderr
            response = json.loads(result.stdout)
            assert response['answers']['next_step']['choice'] == 'measure', response
            assert response['answers']['parity_proven']['noul'] == 0.01, response
            assert response['answers']['evidence_level']['score'] == 0.2, response
            assert len(captured) == 1, captured
            sent = captured[0]
            assert sent['model'] == 'mock-decision-model' and sent['state'] == request['state'], sent
            assert sent['questions'] == request['questions'], sent

            remote_env = os.environ.copy()
            for key in ('GAMEFRAME_DECISION_ENDPOINT', 'GAMEFRAME_DECISION_MODEL',
                        'GAMEFRAME_DECISION_API_KEY', 'GAMEFRAME_LLM_ENDPOINT',
                        'GAMEFRAME_LLM_PROVIDER', 'GAMEFRAME_MODEL'):
                remote_env.pop(key, None)
            remote_env.update({'GAMEFRAME_NO_DOTENV': '1', 'GAMEFRAME_LLM_PROVIDER': 'local',
                               'GAMEFRAME_LLM_ENDPOINT': 'http://127.0.0.1:9/v1',
                               'GAMEFRAME_LOCAL_MODEL': 'unused-model'})
            refused = subprocess.run(
                [sys.executable, str(SCRIPT), str(source), '--model', 'remote-model',
                 '--url', 'https://paid.example/v1/systemone'],
                env=remote_env, input='', capture_output=True, text=True, timeout=10)
            assert refused.returncode == 1 and 'Type yes to continue' in refused.stdout \
                and 'cancelled; no request was sent' in refused.stderr, refused
            assert len(captured) == 1, 'a remote decision request was sent without confirmation'

            invalid = dict(request)
            invalid['questions'] = {'too_few': {'type': 'choice', 'instructions': 'Pick',
                                                  'criteria': {'one': 'Only one option'}}}
            source.write_text(json.dumps(invalid), encoding='utf-8')
            failed = subprocess.run([sys.executable, str(SCRIPT), str(source), '--url', endpoint],
                                    capture_output=True, text=True, timeout=10)
            assert failed.returncode == 1 and '2 to 24 options' in failed.stderr, failed
            assert len(captured) == 1, 'invalid input reached the decision endpoint'
            source.write_text(json.dumps(request), encoding='utf-8')

            routed_env = os.environ.copy()
            for key in ('GAMEFRAME_DECISION_ENDPOINT', 'GAMEFRAME_DECISION_MODEL',
                        'GAMEFRAME_DECISION_API_KEY',
                        'GAMEFRAME_LLM_ENDPOINT', 'GAMEFRAME_LLM_PROVIDER', 'GAMEFRAME_MODEL'):
                routed_env.pop(key, None)
            routed_env.update({
                'GAMEFRAME_NO_DOTENV': '1',
                'GAMEFRAME_LLM_PROVIDER': 'local',
                'GAMEFRAME_LLM_ENDPOINT': 'http://127.0.0.1:9/v1',
                'GAMEFRAME_DECISION_ENDPOINT': endpoint,
                'GAMEFRAME_DECISION_MODEL': 'env-decision-model',
                'GAMEFRAME_DECISION_API_KEY': 'decision-secret',
            })
            routed = subprocess.run([sys.executable, str(SCRIPT), str(source)], env=routed_env,
                                    capture_output=True, text=True, timeout=10)
            assert routed.returncode == 0, routed.stderr
            assert captured[1]['model'] == 'env-decision-model', captured[1]
            assert authorization[1] == 'Bearer decision-secret', authorization

            derived_env = os.environ.copy()
            for key in ('GAMEFRAME_DECISION_ENDPOINT', 'GAMEFRAME_DECISION_MODEL',
                        'GAMEFRAME_DECISION_API_KEY',
                        'GAMEFRAME_LLM_ENDPOINT', 'GAMEFRAME_LLM_PROVIDER', 'GAMEFRAME_MODEL'):
                derived_env.pop(key, None)
            derived_env.update({
                'GAMEFRAME_NO_DOTENV': '1',
                'GAMEFRAME_LLM_PROVIDER': 'local',
                'GAMEFRAME_LLM_ENDPOINT': endpoint.rsplit('/systemone', 1)[0],
                'GAMEFRAME_DECISION_MODEL': 'derived-decision-model',
            })
            derived = subprocess.run([sys.executable, str(SCRIPT), str(source)], env=derived_env,
                                     capture_output=True, text=True, timeout=10)
            assert derived.returncode == 0, derived.stderr
            assert captured[2]['model'] == 'derived-decision-model', captured[2]

            no_fallback_env = os.environ.copy()
            for key in ('GAMEFRAME_DECISION_ENDPOINT', 'GAMEFRAME_DECISION_MODEL',
                        'GAMEFRAME_DECISION_API_KEY',
                        'GAMEFRAME_LLM_ENDPOINT', 'GAMEFRAME_LLM_PROVIDER', 'GAMEFRAME_MODEL'):
                no_fallback_env.pop(key, None)
            no_fallback_env.update({
                'GAMEFRAME_NO_DOTENV': '1',
                'GAMEFRAME_LLM_PROVIDER': 'anthropic',
                'GAMEFRAME_ANTHROPIC_API_KEY': 'saved-service-key',
                'GAMEFRAME_LLM_ENDPOINT': 'https://api.anthropic.com',
            })
            no_fallback = subprocess.run(
                [sys.executable, str(SCRIPT), str(source), '--model', 'env-decision-model'],
                env=no_fallback_env, capture_output=True, text=True, timeout=10)
            assert no_fallback.returncode == 1 and 'Set GAMEFRAME_DECISION_ENDPOINT or pass --url' in no_fallback.stderr, no_fallback
            assert len(captured) == 3, 'a hosted API key or URL selected a decision route'

            help_result = subprocess.run([sys.executable, str(SCRIPT), '--help'],
                                         capture_output=True, text=True, timeout=10)
            assert help_result.returncode == 0 and 'advisory' in help_result.stdout, help_result
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
    print('Decision adapter: bounded questions, explicit route/model/key, derived route, no paid fallback')


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('usage: tests/decision_adapter.py SAMPLE_GAME_EXE')
    main(sys.argv[1])
