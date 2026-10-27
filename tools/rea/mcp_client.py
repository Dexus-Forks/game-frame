"""A small MCP client for one stdio server, for scripts that drive a provider such as REA.

An agent's MCP servers live inside the agent. A script that wants the same provider starts its
own copy as a child process with this client. `provider_command(name)` reads the launch command
from the project's `.mcp.json` (`./gameframe mcp-provider rea VERSION` after
`./gameframe mcp --enable rea`), so a script and the agent run the same version.

    from mcp_client import McpClient, provider_command
    with McpClient(provider_command('rea')) as rea:
        rea.tool('open_binary', path='/abs/game.exe', provider_id='ghidra')

Lines on the server's stdout that are not the JSON-RPC answer are skipped: `npx` prints an
NDJSON notice there before REA starts. `tool()` returns the tool's JSON result and raises
`McpError` when the tool reports an error. Long calls print a line on stderr every 30 s.
"""
import json
import os
import shlex
import subprocess
import sys
import threading
import time

PROTOCOL = '2025-06-18'


class McpError(RuntimeError):
    pass


def project_root():
    """The project the tools work on: $GAMEFRAME_PROJECT (set by ./gameframe), else here."""
    return os.environ.get('GAMEFRAME_PROJECT') or os.getcwd()


def provider_command(name, project=None, override=None):
    """The command of server NAME in the project's .mcp.json, or OVERRIDE (a shell string)."""
    if override:
        return shlex.split(override)
    path = os.path.join(project or project_root(), '.mcp.json')
    try:
        with open(path, encoding='utf-8') as f:
            servers = json.load(f).get('mcpServers', {})
    except FileNotFoundError:
        servers = {}
    if name not in servers:
        raise McpError(f'no {name} server in {path}: run ./gameframe mcp --enable {name}')
    entry = servers[name]
    return [entry['command'], *entry.get('args', [])]


class McpClient:
    """One server process. Use it as a context manager; it stops the server on exit."""

    def __init__(self, command, cwd=None, log=None, start_timeout=300):
        self.log = log or (lambda text: print(text, file=sys.stderr, flush=True))
        self.proc = subprocess.Popen(
            command, cwd=cwd or project_root(), stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, text=True, bufsize=1)
        self.next_id = 0
        self.request('initialize', {
            'protocolVersion': PROTOCOL, 'capabilities': {},
            'clientInfo': {'name': 'gameframe', 'version': '1'}}, start_timeout)
        self._send({'jsonrpc': '2.0', 'method': 'notifications/initialized'})

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        self.stop()

    def stop(self):
        if self.proc.poll() is None:
            self.proc.stdin.close()
            try:
                self.proc.wait(timeout=30)
            except subprocess.TimeoutExpired:
                self.proc.kill()

    def _send(self, message):
        self.proc.stdin.write(json.dumps(message) + '\n')
        self.proc.stdin.flush()

    def _answer(self, want_id, timeout):
        box = {}

        def reader():
            for line in self.proc.stdout:
                try:
                    message = json.loads(line)
                except ValueError:
                    continue
                if isinstance(message, dict) and message.get('id') == want_id:
                    box['message'] = message
                    return

        thread = threading.Thread(target=reader, daemon=True)
        thread.start()
        start = time.monotonic()
        while thread.is_alive():
            thread.join(30)
            waited = time.monotonic() - start
            if thread.is_alive():
                if waited > timeout:
                    raise McpError(f'no answer after {waited:.0f} s')
                self.log(f'  waiting for the server ({waited:.0f} s)')
        if 'message' not in box:
            raise McpError('the server stopped without an answer')
        return box['message']

    def request(self, method, params, timeout=1800):
        self.next_id += 1
        self._send({'jsonrpc': '2.0', 'id': self.next_id, 'method': method, 'params': params})
        message = self._answer(self.next_id, timeout)
        if 'error' in message:
            raise McpError(f"{method}: {message['error'].get('message')}")
        return message['result']

    def tool(self, name, timeout=1800, **arguments):
        """Call one tool and return its result as JSON (REA's `{"result": X}` becomes X)."""
        result = self.request('tools/call', {'name': name, 'arguments': arguments}, timeout)
        if result.get('structuredContent') is not None:
            value = result['structuredContent']
        else:
            text = ''.join(c.get('text', '') for c in result.get('content', []))
            try:
                value = json.loads(text)
            except ValueError:
                value = text
        if result.get('isError'):
            raise McpError(f'{name}: {json.dumps(value)[:600]}')
        if isinstance(value, dict) and set(value) == {'result'}:
            value = value['result']
        return value
