#!/usr/bin/env python3
"""A stand-in for the Claude API and the claude command, so the tests spend nothing.

Usage:
  fake_api.py serve PORTFILE     serve on a free port on 127.0.0.1 and write the port to
                                 PORTFILE; point ANTHROPIC_BASE_URL at http://127.0.0.1:PORT
  fake_api.py claude ARGS...     act as `claude -p --output-format json`: print a JSON result

The server answers POST /v1/messages, and creates, reports and returns Message Batches
(every batch has ended at once). Under a JSON schema (output_config.format) the answer fills
the schema: each string is "fake_<property>", each enum its first value, each number 0 and each
boolean true. Otherwise it echoes
the start of the last user message. The claude mode answers "fake run: " and the start of the
prompt (read from stdin) and the number of CLAUDE* variables it was given, with a cost of 0.01. It writes FAKE_ANSWER, when set, to answer.txt in the current
folder (the tests of skillcheck.py use it).

What it does not do: model anything. It checks the shape of requests only as far as the
tools need, and it keeps batches in memory until it stops.
"""
import json
import os
import sys
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

BATCHES = {}


def fill(schema, key='value'):
    t = schema.get('type')
    if 'enum' in schema:
        return schema['enum'][0]
    if t == 'object':
        return {k: fill(v, k) for k, v in schema.get('properties', {}).items()}
    if t == 'array':
        return [fill(schema.get('items', {}), key)]
    if t in ('integer', 'number'):
        return 0
    if t == 'boolean':
        return True
    return f'fake_{key}'


def answer(params):
    fmt = (params.get('output_config') or {}).get('format')
    if fmt and fmt.get('type') == 'json_schema':
        text = json.dumps(fill(fmt['schema']))
    else:
        last = params['messages'][-1]['content']
        if isinstance(last, list):
            last = ' '.join(b.get('text', '') for b in last)
        text = 'fake answer: ' + last[:60]
    return {'id': 'msg_' + uuid.uuid4().hex[:12], 'type': 'message', 'role': 'assistant',
            'model': params.get('model', 'fake'), 'content': [{'type': 'text', 'text': text}],
            'stop_reason': 'end_turn', 'stop_sequence': None,
            'usage': {'input_tokens': 10, 'output_tokens': 10}}


def batch(bid, host):
    n = len(BATCHES[bid])
    return {'id': bid, 'type': 'message_batch', 'processing_status': 'ended',
            'request_counts': {'processing': 0, 'succeeded': n, 'errored': 0, 'canceled': 0, 'expired': 0},
            'created_at': '2026-01-01T00:00:00Z', 'expires_at': '2026-01-02T00:00:00Z',
            'ended_at': '2026-01-01T00:00:01Z', 'archived_at': None, 'cancel_initiated_at': None,
            'results_url': f'http://{host}/v1/messages/batches/{bid}/results'}


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def send(self, body, kind='application/json'):
        data = body.encode() if isinstance(body, str) else json.dumps(body).encode()
        self.send_response(200)
        self.send_header('Content-Type', kind)
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_POST(self):
        params = json.loads(self.rfile.read(int(self.headers['Content-Length'])) or b'{}')
        if self.path.startswith('/v1/messages/batches'):
            bid = 'msgbatch_' + uuid.uuid4().hex[:12]
            BATCHES[bid] = params['requests']
            self.send(batch(bid, self.headers['Host']))
        else:
            self.send(answer(params))

    def do_GET(self):
        parts = self.path.split('?')[0].strip('/').split('/')
        bid = parts[3] if len(parts) > 3 else ''
        if bid not in BATCHES:
            self.send_error(404)
        elif parts[-1] == 'results':
            lines = [json.dumps({'custom_id': r['custom_id'],
                                 'result': {'type': 'succeeded', 'message': answer(r['params'])}})
                     for r in BATCHES[bid]]
            self.send('\n'.join(lines) + '\n', 'application/binary')
        else:
            self.send(batch(bid, self.headers['Host']))


def main(argv):
    if argv[:1] == ['serve'] and len(argv) == 2:
        server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        open(argv[1], 'w').write(str(server.server_address[1]))
        server.serve_forever()
    elif argv[:1] == ['claude']:
        prompt = sys.stdin.read()
        if os.environ.get('FAKE_ANSWER'):
            open('answer.txt', 'w').write(os.environ['FAKE_ANSWER'])
        print(json.dumps({'type': 'result', 'is_error': False, 'total_cost_usd': 0.01,
                          'result': 'fake run: ' + prompt[:60] + '; CLAUDE variables: '
                                    + str(sum(k.startswith('CLAUDE') for k in os.environ))}))
    else:
        print(__doc__)
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
