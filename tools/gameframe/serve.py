"""GameFrame's MCP server on stdio: the know-how and the search, for clients without shell hooks.

Usage: gameframe serve          (an MCP client starts it; .mcp.json names it "gameframe")

Tools it offers:

  advise(text, signatures_only=false, max=5)  the know-how entries for an error message, a log
                                             or a topic, as `gameframe advise --json` gives them
  knowhow_entry(id)                          one entry in full: its fields and its Markdown body
  knowhow_list()                             every entry with its status and title
  find(word)                                 which skills and tools mention a word, as
                                             `gameframe find --json`

It speaks JSON-RPC 2.0, one message per line, as the MCP stdio transport says: initialize,
ping, tools/list and tools/call. It has no resources or prompts, keeps no state between calls,
and writes nothing. Standard library only.
"""
import json
import sys

import advise
import knowhow
from search import search

VERSION = '2025-06-18'


def _schema(props, required=()):
    return {'type': 'object', 'properties': props, 'required': list(required)}


TOOLS = [
    {'name': 'advise',
     'description': 'Known failure modes, fixes, dead ends and context from GameFrame\'s know-how '
                    '(docs/knowhow/) for an error message, a log or a topic such as "directdraw wine". '
                    'Ask it when a command fails or the work turns to a new era, runtime or hardware.',
     'inputSchema': _schema({'text': {'type': 'string', 'description': 'error text, log lines or topic words'},
                             'signatures_only': {'type': 'boolean', 'description': 'only entries whose error signature matches'},
                             'max': {'type': 'integer', 'description': 'entries returned (default 5)'}}, ('text',))},
    {'name': 'knowhow_entry',
     'description': 'One know-how entry in full: its facets, signatures, links and Markdown body.',
     'inputSchema': _schema({'id': {'type': 'string', 'description': 'the entry id, such as wine-privileged-instruction'}}, ('id',))},
    {'name': 'knowhow_list',
     'description': 'Every know-how entry with its id, status and title.',
     'inputSchema': _schema({})},
    {'name': 'find',
     'description': 'Which GameFrame skills and tools mention a word, line by line.',
     'inputSchema': _schema({'word': {'type': 'string'}}, ('word',))},
]


def call(name, args):
    """(result, is_error) for one tool call."""
    if name == 'advise':
        text = str(args.get('text') or '')
        hits = advise.signature_hits(text) if args.get('signatures_only') else advise.search(text)
        hits = hits[:int(args.get('max') or 5)]
        return [{**h.entry.as_dict(), 'score': h.score, 'why': h.why, 'brief': advise.brief(h.entry)} for h in hits], False
    if name == 'knowhow_entry':
        e = next((e for e in knowhow.entries() if e.id == args.get('id')), None)
        if not e:
            return f'no know-how entry {args.get("id")!r}; knowhow_list names them', True
        return {**e.as_dict(), 'body': e.body}, False
    if name == 'knowhow_list':
        return [{'id': e.id, 'status': e.status, 'title': e.title} for e in knowhow.entries()], False
    if name == 'find':
        return [dict(zip(('kind', 'name', 'where', 'line'), h)) for h in search(str(args.get('word') or ''))], False
    return f'unknown tool {name!r}', True


def handle(msg):
    """The response to one message, or None for a notification."""
    method, mid = msg.get('method'), msg.get('id')
    if mid is None:
        return None
    if method == 'initialize':
        version = (msg.get('params') or {}).get('protocolVersion') or VERSION
        result = {'protocolVersion': version, 'capabilities': {'tools': {}},
                  'serverInfo': {'name': 'gameframe', 'version': '1'},
                  'instructions': 'GameFrame\'s know-how: call advise with the error text when a command fails.'}
    elif method == 'ping':
        result = {}
    elif method == 'tools/list':
        result = {'tools': TOOLS}
    elif method == 'tools/call':
        params = msg.get('params') or {}
        try:
            value, is_error = call(params.get('name'), params.get('arguments') or {})
        except Exception as err:        # report the failure to the client; keep serving
            value, is_error = f'{type(err).__name__}: {err}', True
        text = value if isinstance(value, str) else json.dumps(value, indent=2)
        result = {'content': [{'type': 'text', 'text': text}], 'isError': is_error}
    else:
        return {'jsonrpc': '2.0', 'id': mid, 'error': {'code': -32601, 'message': f'method not found: {method}'}}
    return {'jsonrpc': '2.0', 'id': mid, 'result': result}


def serve(args=None):
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            msg = json.loads(line)
        except ValueError:
            out = {'jsonrpc': '2.0', 'id': None, 'error': {'code': -32700, 'message': 'parse error'}}
        else:
            out = handle(msg) if isinstance(msg, dict) else None
        if out is not None:
            sys.stdout.write(json.dumps(out) + '\n')
            sys.stdout.flush()
    return 0
