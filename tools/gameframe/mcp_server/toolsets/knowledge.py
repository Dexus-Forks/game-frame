"""Know-how and text search tools for GameFrame's MCP server.

Usage: `registry.py` loads `tools()` and registers each definition independently.
Inputs: text, entry IDs and search words.
Outputs: knowledge records and matching skill or tool lines; these handlers only read files.
"""
import advise
import knowhow
from mcp_server.registry import Tool, ToolCallError, schema
from search import search


def _advise(args, _context):
    text = str(args.get('text') or '')
    if not text.strip():
        raise ToolCallError('text must contain an error, log or topic')
    limit = args.get('max', 5)
    if type(limit) is not int or not 1 <= limit <= 50:
        raise ToolCallError('max must be an integer from 1 to 50')
    hits = advise.signature_hits(text) if args.get('signatures_only') else advise.search(text)
    return [{**hit.entry.as_dict(), 'score': hit.score, 'why': hit.why,
             'brief': advise.brief(hit.entry)} for hit in hits[:limit]]


def _entry(args, _context):
    entry_id = args.get('id')
    if not isinstance(entry_id, str) or not entry_id:
        raise ToolCallError('id must be a non-empty string')
    entry = next((item for item in knowhow.entries() if item.id == entry_id), None)
    if entry is None:
        raise ToolCallError(f'no know-how entry {entry_id!r}; use knowhow_list to find IDs')
    return {**entry.as_dict(), 'body': entry.body}


def _list(_args, _context):
    return [{'id': entry.id, 'status': entry.status, 'title': entry.title}
            for entry in knowhow.entries()]


def _find(args, _context):
    word = args.get('word')
    if not isinstance(word, str) or not word.strip():
        raise ToolCallError('word must be a non-empty string')
    return [dict(zip(('kind', 'name', 'where', 'line'), hit)) for hit in search(word)]


def tools():
    return [
        Tool('advise', 'Find known failure modes, fixes, dead ends and context for error text, logs or a topic.',
             schema({'text': {'type': 'string'},
                     'signatures_only': {'type': 'boolean', 'default': False},
                     'max': {'type': 'integer', 'default': 5, 'minimum': 1, 'maximum': 50}}, ('text',)), _advise),
        Tool('knowhow_entry', 'Read one know-how entry with its metadata and Markdown body.',
             schema({'id': {'type': 'string'}}, ('id',)), _entry),
        Tool('knowhow_list', 'List every know-how entry with its status and title.',
             schema(), _list),
        Tool('find', 'Find matching lines in GameFrame skills and tool documentation.',
             schema({'word': {'type': 'string'}}, ('word',)), _find),
    ]
