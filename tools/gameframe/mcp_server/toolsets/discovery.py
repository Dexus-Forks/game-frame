"""Read-only discovery tools for GameFrame's method, skills, agents and tool catalog.

Usage: `registry.py` loads `tools()` and registers each definition independently.
Inputs: optional skill names or progress-log filters.
Outputs: JSON-compatible project metadata and documentation; handlers do not write files.
"""
import os
import time
from pathlib import Path

import agents
import catalog
import progress
import skills
from mcp_server.registry import Tool, ToolCallError, schema
from paths import ROOT, rel


def _steps(_args, _context):
    diagram, rows = skills.steps()
    return {'diagram': diagram,
            'steps': [{'step': step, 'question': question, 'skill': name}
                      for step, question, name in rows]}


def _skills(_args, _context):
    return [item.as_dict() for item in skills.skills()]


def _skill(args, _context):
    name = args.get('name')
    if not isinstance(name, str) or not name:
        raise ToolCallError('name must be a non-empty skill name')
    item, close = skills.find(name)
    if item is None:
        hint = f'; similar names: {", ".join(close)}' if close else ''
        raise ToolCallError(f'no skill named {name!r}{hint}')
    section = args.get('section')
    body = item.body
    if section:
        if not isinstance(section, str):
            raise ToolCallError('section must be a string')
        body = skills.section(item.body, section)
        if body is None:
            raise ToolCallError(f'{item.name} has no section matching {section!r}')
    return {'name': item.name, 'description': item.description, 'path': rel(item.path), 'body': body}


def _agents(_args, _context):
    return [{key: value for key, value in item.items() if key != 'body'}
            for item in agents.agents()]


def _tools(_args, _context):
    return [item.as_dict() for item in catalog.tools()]


def _tool_help(args, _context):
    name = args.get('name')
    if not isinstance(name, str) or not name:
        raise ToolCallError('name must be a non-empty tool name')
    item, close = catalog.find(name)
    if item is None:
        hint = f'; similar names: {", ".join(close)}' if close else ''
        raise ToolCallError(f'no GameFrame tool named {name!r}{hint}')
    return {**item.as_dict(), 'doc': item.doc}


def _progress(args, _context):
    name = args.get('agent')
    if name is not None and (not isinstance(name, str) or not name):
        raise ToolCallError('agent must be a non-empty string when supplied')
    limit = args.get('lines', 10)
    if type(limit) is not int or not 1 <= limit <= 1000:
        raise ToolCallError('lines must be an integer from 1 to 1000')
    paths = progress.logs()
    if name:
        paths = [path for path in paths if Path(path).name == name + '.log']
        if not paths:
            return []
    result = []
    for path in paths:
        log = Path(path)
        lines = log.read_text(encoding='utf-8').splitlines()
        stat = log.stat()
        result.append({'agent': log.stem, 'updated_at': stat.st_mtime,
                       'age_seconds': max(0, int(time.time() - stat.st_mtime)),
                       'lines': lines if name else lines[-limit:]})
    return result


def _providers(_args, _context):
    project = Path(os.environ.get('GAMEFRAME_PROJECT') or os.getcwd()).resolve()
    config = project / '.mcp.json'
    source = 'project'
    if not config.is_file():
        config = ROOT / '.mcp.json'
        source = 'GameFrame defaults'
    if not config.is_file():
        return {'config': str(config), 'source': source, 'providers': []}
    import json
    try:
        servers = json.loads(config.read_text(encoding='utf-8')).get('mcpServers', {})
    except (OSError, ValueError) as err:
        raise ToolCallError(f'cannot read {rel(config)}: {err}')
    found = []
    for name, item in sorted(servers.items()):
        command = item.get('command')
        environment = item.get('env', {})
        arguments = item.get('args', [])
        found.append({'name': name,
                      'command_name': Path(command).name if isinstance(command, str) else None,
                      'argument_count': len(arguments) if isinstance(arguments, list) else None,
                      'environment_keys': sorted(environment) if isinstance(environment, dict) else []})
    return {'config': rel(config), 'source': source, 'providers': found,
            'note': 'This reports configured providers; the MCP client owns their live connections.'}


def tools():
    return [
        Tool('gameframe_steps', 'Read the reverse-engineering workflow and its linked skills.',
             schema(), _steps),
        Tool('gameframe_skills', 'List every GameFrame skill with its description and sections.',
             schema(), _skills),
        Tool('gameframe_skill', 'Read a GameFrame skill, or one section by heading.',
             schema({'name': {'type': 'string'}, 'section': {'type': 'string'}}, ('name',)), _skill),
        Tool('gameframe_agents', 'List GameFrame agents, their skills and their hand-offs.',
             schema(), _agents),
        Tool('gameframe_tool_catalog', 'List every discovered GameFrame tool, script and library.',
             schema(), _tools),
        Tool('gameframe_tool_help', 'Read the usage and limitations documented by one GameFrame tool.',
             schema({'name': {'type': 'string'}}, ('name',)), _tool_help),
        Tool('gameframe_progress', 'Read recent progress logs for agents in this project.',
             schema({'agent': {'type': 'string'},
                     'lines': {'type': 'integer', 'default': 10, 'minimum': 1, 'maximum': 1000}}), _progress),
        Tool('gameframe_mcp_providers', 'List configured MCP provider names and launch metadata without argument or environment values.',
             schema(), _providers),
    ]
