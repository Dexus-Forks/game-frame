"""Serve GameFrame MCP requests over stdio with cancellable tool calls.

Usage: `./gameframe serve` starts one process for one MCP client connection.
Inputs: newline-delimited JSON-RPC messages on stdin.
Outputs: JSON-RPC replies and progress notifications on stdout; diagnostics belong on stderr.
Tool implementations live in `toolsets/`; this module owns transport and request concurrency.
"""
import json
import sys
import threading
import time

from mcp_server.context import RequestContext
from mcp_server.protocol import call_result, error, initialize, response
from mcp_server.registry import build_registry

MAX_ACTIVE_CALLS = 4


def serve(_args=None):
    registry = build_registry()
    output_lock = threading.Lock()
    active_lock = threading.Lock()
    active = {}
    disconnected = threading.Event()

    def write(message):
        if disconnected.is_set():
            return
        with output_lock:
            if not disconnected.is_set():
                sys.stdout.write(json.dumps(message, ensure_ascii=False) + '\n')
                sys.stdout.flush()

    def finish_call(request_id, params, context):
        try:
            name = params.get('name') if isinstance(params, dict) else None
            arguments = params.get('arguments', {}) if isinstance(params, dict) else {}
            value, is_error = registry.call(name, arguments, context)
            if not context.cancelled.is_set():
                write(response(request_id, call_result(value, is_error)))
        finally:
            with active_lock:
                active.pop(request_id, None)

    def start_call(message):
        request_id = message.get('id')
        params = message.get('params') or {}
        meta = params.get('_meta') if isinstance(params, dict) else None
        progress_token = meta.get('progressToken') if isinstance(meta, dict) else None
        if type(progress_token) not in (str, int, float):
            progress_token = None
        context = RequestContext(request_id, progress_token, write)
        with active_lock:
            if request_id in active:
                write(error(request_id, -32600, 'request ID is already active'))
                return
            if len(active) >= MAX_ACTIVE_CALLS:
                write(response(request_id, call_result(
                    f'MCP server supports at most {MAX_ACTIVE_CALLS} simultaneous tool calls', True)))
                return
            thread = threading.Thread(target=finish_call,
                                      args=(request_id, params, context),
                                      name=f'gameframe-mcp-{request_id}', daemon=True)
            active[request_id] = (context, thread)
            thread.start()

    def cancel_call(params):
        if not isinstance(params, dict) or 'requestId' not in params:
            return
        request_id = params['requestId']
        if type(request_id) not in (str, int, float):
            return
        with active_lock:
            call = active.get(request_id)
            if call:
                call[0].cancelled.set()

    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            message = json.loads(line)
        except ValueError:
            write(error(None, -32700, 'parse error'))
            continue
        if not isinstance(message, dict):
            write(error(None, -32600, 'invalid request'))
            continue

        method = message.get('method')
        request_id = message.get('id')
        if request_id is None:
            if method == 'notifications/cancelled':
                cancel_call(message.get('params'))
            continue
        if type(request_id) not in (str, int, float):
            write(error(None, -32600, 'request ID must be a string or number'))
            continue
        if method == 'initialize':
            write(initialize(message))
        elif method == 'ping':
            write(response(request_id, {}))
        elif method == 'tools/list':
            write(response(request_id, {'tools': registry.definitions()}))
        elif method == 'tools/call':
            start_call(message)
        else:
            write(error(request_id, -32601, f'method not found: {method}'))

    # Some simple clients write a batch and close stdin before collecting replies. Give active
    # calls a short drain window so quick discovery calls still return, then cancel leftovers.
    deadline = time.monotonic() + 1.0
    while True:
        with active_lock:
            pending = list(active.values())
        if not pending:
            break
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            disconnected.set()
            for context, _thread in pending:
                context.cancelled.set()
            for _context, thread in pending:
                thread.join()
            break
        for _context, thread in pending:
            thread.join(timeout=min(0.05, remaining))
    return 0
