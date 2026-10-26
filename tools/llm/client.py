#!/usr/bin/env python3
"""Shared clients for GameFrame's model-powered tools (tools/llm/).

Usage:
  from client import api, text_of, run_claude, MODEL, PROTOCOL

Local models are the default. Set GAMEFRAME_LLM_ENDPOINT, GAMEFRAME_LLM_PROTOCOL and the
selected provider's model setting to choose a server, API and model. Use GAMEFRAME_MODEL only
for a one-run override. For hosted use, set
GAMEFRAME_LLM_PROVIDER=anthropic or openai; these named providers use their standard endpoint
and ignore GAMEFRAME_LLM_ENDPOINT. Use openai-compatible with GAMEFRAME_LLM_ENDPOINT for a
custom OpenAI-compatible service. The Anthropic protocol also drives the headless Claude Code
agent.

api() returns a client with messages.create(). Its OpenAI adapter supports text, base64 image
blocks and JSON-schema output. Anthropic batch requests are enabled for the hosted API; set
GAMEFRAME_LLM_BATCHES=1 to enable them on another endpoint that implements that API.

Use GAMEFRAME_LOCAL_LLM_API_KEY for an authenticated local server. Save hosted keys as
GAMEFRAME_ANTHROPIC_API_KEY, GAMEFRAME_OPENAI_API_KEY or
GAMEFRAME_OPENAI_COMPATIBLE_API_KEY. Local mode never sends a hosted key. Set
GAMEFRAME_ANTHROPIC_WORKSPACE_ID when an Anthropic key needs a workspace header. Settings load
from the environment and then .env files in the current folder, project and GameFrame.
GAMEFRAME_NO_DOTENV=1 skips them.

What it does not do: track provider spending or download models. Use each tool's --budget or
--max limits where available. Run this file directly to check the selected setup.
"""
import json
import ipaddress
import os
import shlex
import shutil
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request
from types import SimpleNamespace

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PROJECT = os.environ.get('GAMEFRAME_PROJECT') or ROOT
MODEL_KEYS = {'local': 'GAMEFRAME_LOCAL_MODEL', 'anthropic': 'GAMEFRAME_ANTHROPIC_MODEL',
              'openai': 'GAMEFRAME_OPENAI_MODEL',
              'openai-compatible': 'GAMEFRAME_OPENAI_COMPATIBLE_MODEL'}
API_KEY_KEYS = {'anthropic': 'GAMEFRAME_ANTHROPIC_API_KEY', 'openai': 'GAMEFRAME_OPENAI_API_KEY',
                'openai-compatible': 'GAMEFRAME_OPENAI_COMPATIBLE_API_KEY'}
KEYS = ('GAMEFRAME_LLM_ENDPOINT', 'GAMEFRAME_LLM_PROVIDER', 'GAMEFRAME_LLM_PROTOCOL',
        'GAMEFRAME_LOCAL_LLM_API_KEY', *MODEL_KEYS.values(), *API_KEY_KEYS.values(),
        'GAMEFRAME_ANTHROPIC_WORKSPACE_ID',
        'GAMEFRAME_DECISION_ENDPOINT', 'GAMEFRAME_DECISION_MODEL', 'GAMEFRAME_DECISION_API_KEY',
        'GAMEFRAME_DECISION_DATA_DIR', 'GAMEFRAME_LLM_BATCHES')


def load_env():
    """Read model settings from current, project and GameFrame .env files when unset."""
    if os.environ.get('GAMEFRAME_NO_DOTENV'):
        return
    seen = set()
    for folder in (os.getcwd(), PROJECT, ROOT):
        path = os.path.join(folder, '.env')
        if os.path.realpath(path) in seen or not os.path.exists(path):
            continue
        seen.add(os.path.realpath(path))
        for line in open(path, encoding='utf-8'):
            key, eq, value = line.strip().partition('=')
            if eq and key in KEYS and key not in os.environ:
                os.environ[key] = value.strip().strip('"\'')


load_env()
PROVIDER = os.environ.get('GAMEFRAME_LLM_PROVIDER', 'local').strip().lower()
if PROVIDER not in ('local', 'anthropic', 'openai', 'openai-compatible'):
    sys.exit('GAMEFRAME_LLM_PROVIDER must be local, anthropic, openai or openai-compatible')
if PROVIDER == 'anthropic':
    PROTOCOL = 'anthropic'
elif PROVIDER in ('openai', 'openai-compatible'):
    PROTOCOL = 'openai'
else:
    PROTOCOL = os.environ.get('GAMEFRAME_LLM_PROTOCOL', 'anthropic').strip().lower()
if PROTOCOL not in ('anthropic', 'openai'):
    sys.exit('GAMEFRAME_LLM_PROTOCOL must be anthropic or openai')
CONFIGURED_ENDPOINT = os.environ.get('GAMEFRAME_LLM_ENDPOINT')
# A project may keep its local endpoint in .env. Selecting a named hosted provider must switch
# to that provider's standard endpoint instead of sending its API key to the saved local URL.
BASE_URL = CONFIGURED_ENDPOINT if PROVIDER in ('local', 'openai-compatible') else None
MODEL = os.environ.get('GAMEFRAME_MODEL') or os.environ.get(MODEL_KEYS[PROVIDER])
if not MODEL and PROVIDER == 'anthropic':
    MODEL = 'claude-opus-5-5'
elif not MODEL and PROVIDER == 'openai':
    MODEL = 'gpt-4.1'


class OpenAIMessageClient:
    """Adapt OpenAI-compatible Chat Completions to the Messages API used by these tools."""

    def __init__(self, base_url, api_key):
        self.base_url = (base_url or 'https://api.openai.com/v1').rstrip('/')
        self.api_key = api_key

    @staticmethod
    def _content(value):
        if isinstance(value, str):
            return value
        content = []
        for block in value or []:
            if block.get('type') == 'text':
                content.append({'type': 'text', 'text': block.get('text', '')})
            elif block.get('type') == 'image':
                source = block.get('source', {})
                if source.get('type') != 'base64':
                    raise ValueError('image input must be a base64 image block')
                data_url = f"data:{source['media_type']};base64,{source['data']}"
                content.append({'type': 'image_url', 'image_url': {'url': data_url}})
            else:
                raise ValueError(f"unsupported message content block: {block.get('type')}")
        return content

    def create(self, **params):
        body = {'model': params['model'], 'messages': [], 'max_tokens': params.get('max_tokens', 2048)}
        system = params.get('system')
        if system:
            body['messages'].append({'role': 'system', 'content': system})
        body['messages'].extend({'role': message['role'], 'content': self._content(message['content'])}
                                for message in params.get('messages', []))
        output = params.get('output_config', {}).get('format', {})
        if output.get('type') == 'json_schema':
            body['response_format'] = {
                'type': 'json_schema',
                'json_schema': {'name': 'gameframe_response', 'strict': True,
                                'schema': output['schema']},
            }
        request = urllib.request.Request(
            self.base_url + '/chat/completions',
            data=json.dumps(body, ensure_ascii=False, allow_nan=False).encode('utf-8'),
            headers={'Content-Type': 'application/json', 'Authorization': f'Bearer {self.api_key}'},
            method='POST')
        try:
            with urllib.request.urlopen(request, timeout=600) as response:
                raw = response.read(8 * 1024 * 1024 + 1)
            if len(raw) > 8 * 1024 * 1024:
                raise RuntimeError('OpenAI-compatible response exceeds 8 MiB')
            result = json.loads(raw)
        except urllib.error.HTTPError as error:
            detail = error.read(4096).decode('utf-8', errors='replace')
            raise RuntimeError(f'OpenAI-compatible endpoint returned HTTP {error.code}: {detail}') from error
        except urllib.error.URLError as error:
            raise RuntimeError(f'cannot reach OpenAI-compatible endpoint at {self.base_url}: {error.reason}') from error
        try:
            text = result['choices'][0]['message']['content']
        except (KeyError, IndexError, TypeError) as error:
            raise RuntimeError('OpenAI-compatible response has no assistant message content') from error
        if not isinstance(text, str):
            raise RuntimeError('OpenAI-compatible response content must be text')
        usage = result.get('usage') or {}
        return SimpleNamespace(
            content=[SimpleNamespace(type='text', text=text)],
            usage=SimpleNamespace(input_tokens=usage.get('prompt_tokens', 0),
                                  output_tokens=usage.get('completion_tokens', 0)))


class ModelClient:
    def __init__(self, messages, supports_batches):
        self.messages = MessageAdapter(messages, MODEL)
        self.supports_batches = supports_batches


class MessageAdapter:
    """Require a chosen model and preserve the Anthropic batch interface when present."""

    def __init__(self, messages, default_model):
        self._messages = messages
        self._default_model = default_model
        if hasattr(messages, 'batches'):
            self.batches = messages.batches

    def create(self, **params):
        model = params.get('model') or self._default_model
        if not model:
            sys.exit('Choose a model with --model, a provider profile setting or GAMEFRAME_MODEL.')
        params['model'] = model
        return self._messages.create(**params)


def need_key():
    if PROVIDER == 'local':
        if not BASE_URL:
            sys.exit('Local model use is the default. Set GAMEFRAME_LLM_ENDPOINT to a model server. '
                     'To use a hosted API, choose it by name with GAMEFRAME_LLM_PROVIDER.')
        return
    if PROVIDER == 'openai-compatible' and not BASE_URL:
        sys.exit('Set GAMEFRAME_LLM_ENDPOINT for an OpenAI-compatible provider.')
    key_name = API_KEY_KEYS[PROVIDER]
    if not os.environ.get(key_name):
        sys.exit(f'Set {key_name} for GAMEFRAME_LLM_PROVIDER={PROVIDER}, or select '
                 'GAMEFRAME_LLM_PROVIDER=local and configure GAMEFRAME_LLM_ENDPOINT.')


def selected_api_key():
    if PROVIDER == 'local':
        return os.environ.get('GAMEFRAME_LOCAL_LLM_API_KEY') or 'local'
    return os.environ[API_KEY_KEYS[PROVIDER]]


def _loopback(url):
    if not url:
        return False
    try:
        host = urllib.parse.urlsplit(url).hostname
        return host == 'localhost' or bool(host and ipaddress.ip_address(host).is_loopback)
    except ValueError:
        return False


def confirm_remote_request(url, purpose, force=False):
    """Require a typed yes before a hosted provider or non-loopback endpoint receives data."""
    if not force and _loopback(url):
        return
    host = urllib.parse.urlsplit(url).hostname if url else 'the selected hosted provider'
    print(f'This {purpose} will send request data to {host}; it may incur API charges.', file=sys.stderr)
    try:
        console = 'CONIN$' if os.name == 'nt' else '/dev/tty'
        with open(console, 'r+', encoding='utf-8') as terminal:
            terminal.write('Type yes to continue: ')
            terminal.flush()
            answer = terminal.readline()
    except OSError:
        try:
            answer = input('Type yes to continue: ')
        except EOFError:
            answer = ''
    if answer.strip().lower() != 'yes':
        raise SystemExit('Hosted request cancelled; no request was sent.')


def confirm_provider_use():
    """Confirm hosted providers and local-provider endpoints outside loopback."""
    confirm_remote_request(BASE_URL, f'{PROVIDER} model request', force=PROVIDER != 'local')


def api(confirm=True):
    need_key()
    if confirm:
        confirm_provider_use()
    key = selected_api_key()
    if PROTOCOL == 'openai':
        return ModelClient(OpenAIMessageClient(BASE_URL, key), supports_batches=False)
    try:
        import anthropic
    except ImportError:
        sys.exit('the anthropic package is missing: scripts/setup.sh --install, or '
                 '.venv/bin/pip install -r requirements-llm.txt')
    workspace = os.environ.get('GAMEFRAME_ANTHROPIC_WORKSPACE_ID')
    client = anthropic.Anthropic(api_key=key, base_url=BASE_URL,
                                 default_headers={'anthropic-workspace-id': workspace} if workspace else None)
    batches = not BASE_URL or os.environ.get('GAMEFRAME_LLM_BATCHES') == '1'
    return ModelClient(client.messages, supports_batches=batches)


def text_of(msg):
    return ''.join(b.text for b in msg.content if b.type == 'text')


def claude_command():
    if os.environ.get('GAMEFRAME_CLAUDE'):
        return shlex.split(os.environ['GAMEFRAME_CLAUDE'])
    if not shutil.which('claude'):
        sys.exit('the claude command is missing: install Claude Code')
    return ['claude']


def run_claude(prompt, system=None, cwd=None, tools='Read,Glob,Grep,Bash', budget=2.0,
               model=None, timeout=3600):
    """Run one headless Claude Code agent through an Anthropic-compatible provider."""
    if PROTOCOL != 'anthropic':
        raise RuntimeError('Claude Code needs an Anthropic-compatible endpoint. Set '
                           'GAMEFRAME_LLM_PROTOCOL=anthropic; Ollama, LM Studio and llama.cpp support it.')
    need_key()
    confirm_provider_use()
    model = model or MODEL
    if not model:
        sys.exit('Choose an agent model with --model, a provider profile setting or GAMEFRAME_MODEL.')
    cmd = claude_command() + ['-p', '--bare', '--output-format', 'json', '--no-session-persistence',
                              '--model', model, '--max-budget-usd', str(budget),
                              '--permission-mode', 'bypassPermissions', '--tools', tools]
    if system:
        cmd += ['--system-prompt', system]
    # A run started inside Claude Code must not join the user's live session.
    native_keys = {'ANTHROPIC_API_KEY', 'ANTHROPIC_AUTH_TOKEN', 'ANTHROPIC_BASE_URL',
                   'ANTHROPIC_WORKSPACE_ID', 'ANTHROPIC_CUSTOM_HEADERS'}
    env = {k: v for k, v in os.environ.items()
           if not k.startswith('CLAUDE') and k not in native_keys}
    if BASE_URL:
        env['ANTHROPIC_BASE_URL'] = BASE_URL
    token = selected_api_key()
    env['ANTHROPIC_API_KEY'] = token
    env['ANTHROPIC_AUTH_TOKEN'] = token
    workspace = os.environ.get('GAMEFRAME_ANTHROPIC_WORKSPACE_ID')
    if workspace:
        env['ANTHROPIC_WORKSPACE_ID'] = workspace
        env['ANTHROPIC_CUSTOM_HEADERS'] = f'anthropic-workspace-id: {workspace}'
    run = subprocess.run(cmd, input=prompt, cwd=cwd, env=env, capture_output=True, text=True, timeout=timeout)
    try:
        return json.loads(run.stdout)
    except json.JSONDecodeError:
        return {'is_error': True, 'result': (run.stdout + run.stderr).strip()[-2000:], 'total_cost_usd': 0}


def main():
    ok = True
    configured_key = (os.environ.get('GAMEFRAME_LOCAL_LLM_API_KEY') or BASE_URL
                      if PROVIDER == 'local' else os.environ.get(API_KEY_KEYS[PROVIDER]))
    print(f'provider: {PROVIDER}')
    if PROTOCOL == 'anthropic':
        try:
            import anthropic
            print(f'anthropic {anthropic.__version__}')
        except ImportError:
            print('anthropic: missing (scripts/setup.sh --install)')
            ok = False
        print('API key or endpoint: ' + ('set' if configured_key else 'missing'))
    else:
        print('OpenAI-compatible protocol (standard library client)')
        print('API key or endpoint: ' + ('set' if configured_key else 'missing'))
        ok = (PROVIDER == 'local' and bool(BASE_URL)
              or PROVIDER != 'local' and bool(configured_key))
    print('claude: ' + (' '.join(claude_command()) if os.environ.get('GAMEFRAME_CLAUDE')
                        or shutil.which('claude') else 'missing'))
    print(f'protocol: {PROTOCOL}; model: {MODEL or "not selected"}'
          + (f'; endpoint: {BASE_URL}' if BASE_URL else ''))
    if not MODEL:
        print('model: set the selected provider\'s model setting, GAMEFRAME_MODEL or --model')
        ok = False
    if PROVIDER == 'local' and not BASE_URL:
        print('endpoint: set GAMEFRAME_LLM_ENDPOINT to a local model server')
        ok = False
    if PROVIDER != 'local':
        if PROVIDER == 'openai-compatible' and not BASE_URL:
            print('endpoint: set GAMEFRAME_LLM_ENDPOINT for an OpenAI-compatible provider')
            ok = False
        ok = ok and bool(configured_key)
    return 0 if ok else 1


if __name__ == '__main__':
    if sys.argv[1:] in (['-h'], ['--help']):
        print(__doc__)
        sys.exit(0)
    sys.exit(main())
