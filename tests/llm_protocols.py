#!/usr/bin/env python3
"""Exercise the OpenAI-compatible client, vision input and synchronous local naming path."""
import json
import os
import subprocess
import sys
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, 'tools', 'llm'))
from fake_api import fill  # noqa: E402

SEEN = []
AUTH = []


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_args):
        pass

    def do_POST(self):
        params = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        SEEN.append(params)
        AUTH.append(self.headers.get('Authorization'))
        schema = ((params.get('response_format') or {}).get('json_schema') or {}).get('schema')
        text = json.dumps(fill(schema)) if schema else 'local fake answer'
        data = json.dumps({'choices': [{'message': {'role': 'assistant', 'content': text}}],
                           'usage': {'prompt_tokens': 13, 'completion_tokens': 7}}).encode()
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def main():
    server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    temporary = tempfile.TemporaryDirectory(prefix='gameframe-local-model-')
    root = temporary.name
    os.environ.update({'GAMEFRAME_NO_DOTENV': '1', 'GAMEFRAME_PROJECT': root,
                       'GAMEFRAME_LLM_PROVIDER': 'local',
                       'GAMEFRAME_LLM_PROTOCOL': 'openai',
                       'GAMEFRAME_LLM_ENDPOINT': f'http://127.0.0.1:{server.server_address[1]}/v1',
                       'GAMEFRAME_LOCAL_LLM_API_KEY': 'local-only-key',
                       'GAMEFRAME_ANTHROPIC_API_KEY': 'hosted-key-must-not-leak',
                       'GAMEFRAME_MODEL': 'sample-vision-model'})
    try:
        import client
        assert client.PROVIDER == 'local', client.PROVIDER
        msg = client.api().messages.create(
            model=client.MODEL, max_tokens=128, system='Inspect the supplied screen.',
            output_config={'format': {'type': 'json_schema', 'schema': {
                'type': 'object', 'properties': {'found': {'type': 'boolean'}},
                'required': ['found'], 'additionalProperties': False}}},
            messages=[{'role': 'user', 'content': [
                {'type': 'text', 'text': 'Find the button.'},
                {'type': 'image', 'source': {'type': 'base64', 'media_type': 'image/png', 'data': 'aGVsbG8='}},
            ]}])
        assert json.loads(client.text_of(msg)) == {'found': True}
        sent = SEEN[-1]
        assert AUTH[-1] == 'Bearer local-only-key', AUTH[-1]
        assert sent['model'] == 'sample-vision-model'
        assert sent['response_format']['type'] == 'json_schema'
        assert sent['messages'][1]['content'][1]['image_url']['url'] == 'data:image/png;base64,aGVsbG8='
        assert msg.usage.input_tokens == 13 and msg.usage.output_tokens == 7

        # locate.py uses the same message adapter for image questions and parses its schema result.
        import locate
        result = locate.ask([{'type': 'text', 'text': 'Find the Exit button.'}], locate.FIND, client.MODEL)
        assert result == {'found': True, 'x': 0, 'y': 0, 'note': 'fake_note'}

        # The Anthropic Batch API is absent here, so function naming executes and saves local calls.
        import name_functions
        out = name_functions.folder('demo.exe')
        os.makedirs(out)
        with open(os.path.join(out, 'requests.jsonl'), 'w', encoding='utf-8') as f:
            for address in ('00401000', '00401040'):
                f.write(json.dumps({'custom_id': 'FUN_' + address, 'text': 'sample function'}) + '\n')
        name_functions.submit('demo.exe', client.MODEL, 2)
        name_functions.status('demo.exe')
        name_functions.fetch('demo.exe')
        name_functions.cancel('demo.exe')
        suggestions = open(os.path.join(out, 'suggested.tsv'), encoding='utf-8').read()
        assert '00401000\tfake_name\tlow' in suggestions and '00401040\tfake_name\tlow' in suggestions
        assert len([p for p in SEEN if p.get('response_format')]) >= 3
        try:
            client.run_claude('local agent test')
        except RuntimeError as error:
            assert 'Anthropic-compatible endpoint' in str(error)
        else:
            raise AssertionError('Claude Code must require the Anthropic protocol')

        # A saved local URL must not receive a hosted provider key when a task switches provider.
        hosted_env = os.environ.copy()
        hosted_env.pop('GAMEFRAME_MODEL', None)
        hosted_env.update({'GAMEFRAME_LLM_PROVIDER': 'anthropic',
                           'GAMEFRAME_ANTHROPIC_API_KEY': 'hosted-only-key',
                           'GAMEFRAME_LOCAL_MODEL': 'local-profile-model',
                           'GAMEFRAME_ANTHROPIC_MODEL': 'hosted-profile-model'})
        check = subprocess.run(
            [sys.executable, '-c',
             "import client; assert client.PROVIDER == 'anthropic'; "
             "assert client.PROTOCOL == 'anthropic'; assert client.BASE_URL is None; "
             "assert client.MODEL == 'hosted-profile-model'; "
             "assert client.selected_api_key() == 'hosted-only-key'; "
             "client.confirm_provider_use()"],
            cwd=os.path.join(ROOT, 'tools', 'llm'), env=hosted_env,
            input='no\n', capture_output=True, text=True)
        assert check.returncode != 0 and 'Type yes to continue' in check.stdout \
            and 'cancelled; no request was sent' in check.stderr, check
        allowed = subprocess.run(
            [sys.executable, '-c', 'import client; client.confirm_provider_use()'],
            cwd=os.path.join(ROOT, 'tools', 'llm'), env=hosted_env,
            input='yes\n', capture_output=True, text=True)
        assert allowed.returncode == 0 and 'may incur API charges' in allowed.stderr, allowed
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=2)
        temporary.cleanup()
    print('OpenAI-compatible calls: text, schema, images, synchronous naming, key isolation, hosted confirmation')


if __name__ == '__main__':
    main()
