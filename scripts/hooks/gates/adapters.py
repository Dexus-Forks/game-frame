"""Turn one agent's pre-tool hook event into Check records for checks.reason().

Each adapter takes the event (the JSON the agent writes on the hook's stdin) and returns a list
of Check records, empty when the tool call is none of a commit, a push, a file write or a shell
command that may delete files (the delete gate, deletes.py). The
field names follow each agent's own documentation, as of October 2026:

- Claude Code (PreToolUse): tool_input.command for Bash, checked for the gate's git verb because
  the hook's `if` filter also matches commands it cannot parse; tool_input.file_path with content
  (Write) or new_string (Edit); agent_type names the subagent.
- Codex (PreToolUse): tool_name Bash with tool_input.command; tool_name apply_patch with the
  patch text in tool_input.command (*** Add File: / *** Update File: headers, + lines).
- Gemini CLI (BeforeTool): run_shell_command with tool_input.command and dir_path; write_file
  with file_path and content; replace with file_path and new_string.
- Cursor (beforeShellExecution): command and cwd. Also preToolUse with tool_name Shell and
  tool_input.command. Cursor documents no file fields before a write, so writes are not checked.
- GitHub Copilot (preToolUse): toolName bash with toolArgs, a JSON string holding command; the
  VS Code form tool_name and tool_input too. Copilot documents no fields for edit and create, so
  writes are not checked.

Outside Claude Code the agent comes from GAMEFRAME_AGENT in the environment, or a leading
GAMEFRAME_AGENT=re-name on the command itself.
"""
import json
import os

from checks import Check, agent_name, command_agent, env_agent, git_verbs
from deletes import MAY_DELETE

AGENTS = ('claude', 'codex', 'gemini', 'cursor', 'copilot')


def _cwd(event):
    roots = event.get('workspace_roots') or []
    return event.get('cwd') or (roots[0] if roots else None) or os.getcwd()


def shell(command, cwd, agent=None):
    """Commit, push and delete checks for one shell command."""
    who = agent or env_agent() or command_agent(command)
    found = [Check(v, cwd, who, command=command) for v in git_verbs(command)]
    if MAY_DELETE.search(command or ''):
        found.append(Check('delete', cwd, who, command=command))
    return found


def claude(event, mode):
    """mode is the gate the hook was wired to: commit, push, write or delete."""
    tool, cwd = event.get('tool_input') or {}, _cwd(event)
    if mode == 'delete':
        command = tool.get('command', '')
        return [Check('delete', cwd, command=command)] if MAY_DELETE.search(command) else []
    if mode == 'write':
        return [Check('write', cwd, path=tool.get('file_path', ''),
                      text=tool.get('content') or tool.get('new_string') or '')]
    command = tool.get('command', '')
    # Claude Code's `if: Bash(git commit *)` also matches commands it cannot parse (a heredoc, a
    # shell variable, a for loop), so check that the command really runs the verb.
    if mode not in git_verbs(command):
        return []
    who = agent_name(event.get('agent_type')) or env_agent() or command_agent(command)
    return [Check(mode, cwd, who, command=command)]


def patch_writes(patch, cwd):
    """Write checks for a Codex apply_patch text: the added lines of each file."""
    checks, current = [], None
    for line in patch.splitlines():
        for mark in ('*** Add File: ', '*** Update File: ', '*** Move to: '):
            if line.strip().startswith(mark):
                current = Check('write', cwd, path=line.strip()[len(mark):].strip())
                checks.append(current)
                break
        else:
            if current and line.startswith('+'):
                current.text += line[1:] + '\n'
    return checks


def codex(event):
    tool, cwd = event.get('tool_input') or {}, _cwd(event)
    name = event.get('tool_name')
    if name == 'Bash':
        return shell(tool.get('command', ''), cwd)
    if name == 'apply_patch':
        return patch_writes(tool.get('command', ''), cwd)
    return []


def gemini(event):
    tool, cwd = event.get('tool_input') or {}, _cwd(event)
    name = event.get('tool_name')
    if name == 'run_shell_command':
        return shell(tool.get('command', ''), os.path.join(cwd, tool.get('dir_path') or ''))
    if name == 'write_file':
        return [Check('write', cwd, path=tool.get('file_path', ''), text=tool.get('content') or '')]
    if name == 'replace':
        return [Check('write', cwd, path=tool.get('file_path', ''), text=tool.get('new_string') or '')]
    return []


def cursor(event):
    if 'tool_name' in event:  # preToolUse
        tool = event.get('tool_input') or {}
        if event['tool_name'] != 'Shell':
            return []
        return shell(tool.get('command', ''), tool.get('working_directory') or _cwd(event))
    return shell(event.get('command', ''), _cwd(event))  # beforeShellExecution


def copilot(event):
    name = event.get('toolName') or event.get('tool_name')
    args = event.get('toolArgs', event.get('tool_input')) or {}
    if isinstance(args, str):
        try:
            args = json.loads(args)
        except ValueError:
            args = {}
    if name != 'bash' or not isinstance(args, dict):
        return []
    return shell(args.get('command', ''), _cwd(event))


def checks_for(agent, event, mode=None):
    if agent == 'claude':
        return claude(event, mode)
    return {'codex': codex, 'gemini': gemini, 'cursor': cursor, 'copilot': copilot}[agent](event)
