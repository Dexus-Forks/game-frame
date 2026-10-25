"""JSON-RPC and MCP response helpers for the stdio server.

Usage: `server.py` uses these helpers to answer one request or emit one notification.
Inputs: JSON-RPC IDs, methods, MCP tool definitions and tool values.
Outputs: JSON-serializable protocol messages; this module does not read or write streams.
"""
import json

VERSION = '2025-06-18'


def response(request_id, result):
    return {'jsonrpc': '2.0', 'id': request_id, 'result': result}


def error(request_id, code, message):
    return {'jsonrpc': '2.0', 'id': request_id,
            'error': {'code': code, 'message': message}}


def initialize(request):
    params = request.get('params')
    if not isinstance(params, dict):
        params = {}
    version = params.get('protocolVersion')
    if not isinstance(version, str) or not version:
        version = VERSION
    return response(request.get('id'), {
        'protocolVersion': version,
        'capabilities': {'tools': {}},
        'serverInfo': {'name': 'gameframe', 'version': '2'},
        'instructions': ('Use gameframe_tool_catalog and gameframe_tool_help to discover local '
                         'tools, then gameframe_run_tool to call a documented executable tool. '
                         'External providers are listed in the project MCP configuration.')})


def call_result(value, is_error=False):
    text = value if isinstance(value, str) else json.dumps(value, indent=2, ensure_ascii=False)
    return {'content': [{'type': 'text', 'text': text}], 'isError': is_error}
