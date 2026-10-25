"""Run documented GameFrame programs and scripts through MCP.

Usage: call `gameframe_tool_catalog` and `gameframe_tool_help`, then `gameframe_run_tool`.
Inputs: a catalog name, an argument array, a timeout and an output-size limit.
Outputs: command, working directory, exit status, captured output and timeout/cancel state.
This runs only catalog entries marked runnable and never invokes a shell command string.
"""
import os
import signal
import subprocess
import tempfile
import time

import catalog
from mcp_server.registry import Tool, ToolCallError, ToolOutput, schema
from paths import python

MAX_TIMEOUT = 3600
DEFAULT_TIMEOUT = 120
DEFAULT_OUTPUT = 200_000
MAX_OUTPUT = 1_000_000
PROGRESS_INTERVAL = 1.0


def _stop(process):
    if os.name == 'posix':
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            try:
                os.killpg(process.pid, 0)
            except ProcessLookupError:
                break
            if process.poll() is None:
                try:
                    process.wait(timeout=0.05)
                except subprocess.TimeoutExpired:
                    pass
            else:
                time.sleep(0.05)
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        if process.poll() is None:
            process.wait()
    else:
        if process.poll() is not None:
            return
        try:
            process.terminate()
        except ProcessLookupError:
            return
    try:
        process.wait(timeout=2)
    except subprocess.TimeoutExpired:
        if os.name == 'posix':
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        else:
            try:
                process.kill()
            except ProcessLookupError:
                pass
        process.wait()


def _read_output(stream, maximum):
    size = stream.tell()
    stream.flush()
    stream.seek(0)
    if size <= maximum:
        return stream.read().decode('utf-8', errors='replace'), False
    head_size = maximum // 2
    tail_size = maximum - head_size
    head = stream.read(head_size).decode('utf-8', errors='replace')
    stream.seek(-tail_size, os.SEEK_END)
    tail = stream.read(tail_size).decode('utf-8', errors='replace')
    middle = f'\n... {size - maximum} bytes omitted ...\n'
    return head + middle + tail, True


def _arguments(args):
    name = args.get('name')
    if not isinstance(name, str) or not name:
        raise ToolCallError('name must be a non-empty GameFrame tool name')
    values = args.get('args', [])
    if not isinstance(values, list) or any(not isinstance(value, str) for value in values):
        raise ToolCallError('args must be an array of strings, one item per command-line argument')
    timeout = args.get('timeout_seconds', DEFAULT_TIMEOUT)
    if type(timeout) is not int or not 1 <= timeout <= MAX_TIMEOUT:
        raise ToolCallError(f'timeout_seconds must be an integer from 1 to {MAX_TIMEOUT}')
    maximum = args.get('max_output_bytes', DEFAULT_OUTPUT)
    if type(maximum) is not int or not 4096 <= maximum <= MAX_OUTPUT:
        raise ToolCallError(f'max_output_bytes must be an integer from 4096 to {MAX_OUTPUT}')
    return name, values, timeout, maximum


def _run(args, context):
    name, values, timeout, maximum = _arguments(args)
    item, close = catalog.find(name)
    if item is None:
        hint = f'; similar names: {", ".join(close)}' if close else ''
        raise ToolCallError(f'no GameFrame tool named {name!r}{hint}; use gameframe_tool_catalog')
    if not item.runs:
        route = ('Use `ghidra-run` or `ghidra-export` for Ghidra scripts.'
                 if item.kind == 'ghidra' else 'This is a library; use its documented Python API.')
        raise ToolCallError(f'{item.name} is not directly runnable. {route}')

    py, note = python()
    if item.kind != 'program':
        note = ''
    command = catalog.command(item, py, values)
    project = os.environ.get('GAMEFRAME_PROJECT') or os.getcwd()
    working_directory = os.path.realpath(project)
    if not os.path.isdir(working_directory):
        raise ToolCallError(f'project working directory does not exist: {working_directory}')
    context.check_cancelled()

    started = time.monotonic()
    status = 'completed'
    with tempfile.TemporaryFile() as stdout_file, tempfile.TemporaryFile() as stderr_file:
        try:
            process = subprocess.Popen(command, cwd=working_directory, env=os.environ.copy(),
                                       stdin=subprocess.DEVNULL, stdout=stdout_file,
                                       stderr=stderr_file, start_new_session=os.name == 'posix')
        except OSError as err:
            raise ToolCallError(f'could not start {item.name}: {err}') from err
        next_progress = started
        try:
            context.report_progress(0, timeout, f'Started {item.name}')
            while process.poll() is None:
                if context.cancelled.is_set():
                    status = 'cancelled'
                    _stop(process)
                    break
                elapsed = time.monotonic() - started
                if elapsed >= timeout:
                    status = 'timed_out'
                    _stop(process)
                    break
                now = time.monotonic()
                if now - next_progress >= PROGRESS_INTERVAL:
                    context.report_progress(min(elapsed, timeout), timeout,
                                            f'{item.name} is still running')
                    next_progress = now
                try:
                    process.wait(timeout=min(0.2, max(0.01, timeout - elapsed)))
                except subprocess.TimeoutExpired:
                    pass
        finally:
            if process.poll() is None:
                _stop(process)

        elapsed = round(time.monotonic() - started, 3)
        stdout, stdout_truncated = _read_output(stdout_file, maximum)
        stderr, stderr_truncated = _read_output(stderr_file, maximum)
        result = {
            'tool': item.name,
            'tool_path': item.as_dict()['path'],
            'arguments': values,
            'working_directory': working_directory,
            'status': status,
            'exit_code': process.returncode,
            'duration_seconds': elapsed,
            'stdout': stdout,
            'stderr': stderr,
            'stdout_truncated': stdout_truncated,
            'stderr_truncated': stderr_truncated,
        }
        if note:
            result['environment_note'] = note
        if status == 'completed':
            context.report_progress(timeout, timeout, f'{item.name} finished')
        return ToolOutput(result, status != 'completed' or process.returncode != 0)


def tools():
    return [Tool(
        'gameframe_run_tool',
        'Run one documented GameFrame program or script. Discover names with gameframe_tool_catalog, '
        'read exact usage with gameframe_tool_help, and pass each CLI argument as one string. '
        'The command may write files as documented by that tool. Long runs report progress and can '
        'be cancelled by the MCP client.',
        schema({'name': {'type': 'string', 'description': 'Catalog name, such as peinfo or ghidra-export'},
                'args': {'type': 'array', 'items': {'type': 'string'},
                         'description': 'One array item per command-line argument; no shell parsing is used'},
                'timeout_seconds': {'type': 'integer', 'default': DEFAULT_TIMEOUT,
                                    'minimum': 1, 'maximum': MAX_TIMEOUT},
                'max_output_bytes': {'type': 'integer', 'default': DEFAULT_OUTPUT,
                                     'minimum': 4096, 'maximum': MAX_OUTPUT}}, ('name',)), _run)]
