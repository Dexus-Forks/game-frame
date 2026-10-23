"""Read one agent's post-tool hook event as a shell run. Write the know-how note as its reply.

read(agent, event) returns a Run (command, output, cwd, session). It returns None when the tool
call was not a shell command. reply(agent, event, text) returns the JSON the agent reads to add
the text to its context. The field names follow each agent's own documentation, as of October
2026:

- Claude Code (PostToolUse, PostToolUseFailure): tool_input.command; tool_response with stdout
  and stderr, or error after a failure. Reply: hookSpecificOutput.additionalContext.
- Codex (PostToolUse, which also runs after a non-zero exit): tool_name Bash, tool_input.command,
  tool_response. Reply: the same as Claude Code's.
- Gemini CLI (AfterTool): tool_name run_shell_command, tool_input.command, tool_response with
  llmContent, returnDisplay and error. Reply: hookSpecificOutput.additionalContext.
- Cursor (postToolUse): tool_name Shell, tool_input.command, tool_output (a JSON string).
  Reply: additional_context. Its afterShellExecution event cannot add context. The hook does
  not use it.
- GitHub Copilot (postToolUse, postToolUseFailure): toolName bash, toolArgs (a JSON string with
  command), toolResult.textResultForLlm or error. The VS Code form uses tool_name, tool_input,
  tool_result.text_result_for_llm. Reply: additionalContext.
"""
import json
import os
from dataclasses import dataclass

AGENTS = ('claude', 'codex', 'gemini', 'cursor', 'copilot')
SHELLS = {'claude': ('Bash',), 'codex': ('Bash',), 'gemini': ('run_shell_command',),
          'cursor': ('Shell', 'run_terminal_cmd'), 'copilot': ('bash', 'shell', 'powershell')}


@dataclass
class Run:
    command: str
    output: str
    cwd: str
    session: str


def _text(value):
    """Every string inside a value, joined: a tool response can be a string, a dict or a list."""
    if value is None:
        return ''
    if isinstance(value, str):
        try:
            parsed = json.loads(value)
        except ValueError:
            return value
        return _text(parsed) if isinstance(parsed, (dict, list)) else value
    if isinstance(value, dict):
        return '\n'.join(t for t in (_text(v) for v in value.values()) if t)
    if isinstance(value, list):
        return '\n'.join(t for t in (_text(v) for v in value) if t)
    return ''


def _args(value):
    if isinstance(value, str):
        try:
            value = json.loads(value)
        except ValueError:
            return {}
    return value if isinstance(value, dict) else {}


def read(agent, event):
    """The shell run in this event, or None."""
    name = event.get('tool_name') or event.get('toolName')
    if name and name not in SHELLS[agent]:
        return None
    tool = _args(event.get('tool_input') if 'tool_input' in event else event.get('toolArgs'))
    result = [event.get(k) for k in ('tool_response', 'tool_output', 'toolResult', 'tool_result', 'error')]
    roots = event.get('workspace_roots') or []
    cwd = event.get('cwd') or (roots[0] if roots else None) or os.getcwd()
    session = (event.get('session_id') or event.get('sessionId') or event.get('conversation_id')
               or event.get('transcript_path') or '')
    return Run(tool.get('command', '') or '', '\n'.join(t for t in map(_text, result) if t), cwd, str(session))


def reply(agent, event, text):
    """The JSON that adds `text` to the agent's context."""
    if agent == 'cursor':
        return {'additional_context': text}
    if agent == 'copilot':
        return {'additionalContext': text}
    default = 'AfterTool' if agent == 'gemini' else 'PostToolUse'
    return {'hookSpecificOutput': {'hookEventName': event.get('hook_event_name') or default,
                                   'additionalContext': text}}
