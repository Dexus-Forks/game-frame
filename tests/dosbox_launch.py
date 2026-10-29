#!/usr/bin/env python3
"""Check DOSBox-X launch refusal without starting an emulator.

Usage: python3 tests/dosbox_launch.py
Inputs: our built DOS sample, controlled process lists and a recording executable.
Outputs: fails if a busy or unknown process state permits a launch or changes files.
It does not connect to, launch or stop a real DOSBox-X process.
"""
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SAMPLE = ROOT / 'examples/dos-sample/build/sample-mz.exe'


def executable(path, text):
    path.write_text(text)
    path.chmod(0o755)


def tree(path):
    return {str(p.relative_to(path)): None if p.is_dir() else p.read_bytes()
            for p in path.rglob('*')}


def check(name, processes, *, options=(), busy=False, ps_error=False,
          existing=False, binary='custom-dosbox-x', pids=(), platform=None,
          proc_exe='', readlink_error=False, symlink_target=None):
    with tempfile.TemporaryDirectory(prefix='gameframe-dosbox-launch-') as temp:
        project = Path(temp)
        (project / '.env').write_text('')
        bins = project / 'bin'
        bins.mkdir()
        listing = project / 'processes.txt'
        listing.write_text(processes)
        executable(bins / 'ps', '#!/bin/sh\n'
                   'if [ "$DOSBOX_TEST_PS_ERROR" = 1 ]; then exit 1; fi\n'
                   'cat "$DOSBOX_TEST_PS"\n')
        if platform:
            executable(bins / 'uname', '#!/bin/sh\nprintf "%s\\n" "$DOSBOX_TEST_PLATFORM"\n')
            executable(bins / 'readlink', '#!/bin/sh\n'
                       'for arg do last="$arg"; done\n'
                       '[ "$last" = /proc/4244/exe ] || exit 1\n'
                       '[ "$DOSBOX_TEST_READLINK_ERROR" = 0 ] || exit 1\n'
                       'printf "%s\\n" "$DOSBOX_TEST_EXE"\n')
        emulator = bins / binary
        recording = bins / (symlink_target or binary)
        executable(recording, '#!/bin/sh\n'
                   'printf "%s\\n" "$*" >> "$DOSBOX_TEST_CALLS"\n'
                   'if [ "${1:-}" = -version ]; then echo "DOSBox-X version 2026.10.01"; fi\n')
        if symlink_target:
            emulator.symlink_to(recording)
            proc_exe = str(recording)
        game = project / 'game'
        game.mkdir()
        shutil.copyfile(SAMPLE, game / 'SAMPLEMZ.EXE')
        out = project / 'out'
        if existing:
            (out / 'drive').mkdir(parents=True)
            for path in (game / 'RUN_RE.BAT', game / 'EXITCODE.TXT', out / 'dosbox.log',
                         out / 'exitcode.txt', out / 'drive/keep.txt'):
                path.write_text('keep this existing file\n')
        calls = project / 'calls.txt'
        env = {**os.environ, 'GAMEFRAME_PROJECT': str(project),
               'GAMEFRAME_HOME': str(project / 'home'), 'DOSBOX_X_BINARY': str(emulator),
               'PATH': str(bins) + os.pathsep + os.environ['PATH'],
               'DOSBOX_TEST_PS': str(listing), 'DOSBOX_TEST_PS_ERROR': str(int(ps_error)),
               'DOSBOX_TEST_PLATFORM': platform or '', 'DOSBOX_TEST_EXE': proc_exe,
               'DOSBOX_TEST_READLINK_ERROR': str(int(readlink_error)),
               'DOSBOX_TEST_CALLS': str(calls)}
        before = tree(project)
        result = subprocess.run(['bash', str(ROOT / 'scripts/dosbox-run.sh'), '--out', str(out),
                                 *options, str(game), 'SAMPLEMZ.EXE'], env=env, cwd=ROOT,
                                text=True, capture_output=True, timeout=10)
        if busy or ps_error or readlink_error:
            expected = 75 if busy else 2
            assert result.returncode == expected, (name, result.returncode, result.stdout, result.stderr)
            assert not calls.exists(), (name, 'the emulator was invoked')
            assert tree(project) == before, (name, 'files changed despite launch refusal')
            if busy:
                assert 'already running' in result.stderr.lower(), (name, result.stderr)
                for pid in pids:
                    assert str(pid) in result.stderr, (name, pid, result.stderr)
            else:
                assert result.stderr.strip(), (name, 'process-query failure needs a diagnostic')
        else:
            assert result.returncode == 0, (name, result.stdout, result.stderr)
            assert calls.is_file() and '-conf' in calls.read_text(), (name, 'launch did not proceed')
            assert (out / 'run.conf').is_file(), (name, 'run configuration was not written')


def main():
    assert SAMPLE.is_file(), 'build examples/dos-sample first'
    busy = ' 4242 /Applications/DOSBox-X.app/Contents/MacOS/DOSBox-X\n 4243 dosbox-x\n'
    check('busy before output creation', busy, busy=True, pids=(4242, 4243))
    check('busy before MCP version probe', busy, options=('--mcp', '58991'),
          busy=True, pids=(4242, 4243))
    check('busy preserves in-place files and existing output', busy,
          options=('--in-place',), busy=True, existing=True, pids=(4242, 4243))
    check('configured renamed binary', ' 4244 /opt/custom/renamed-emulator\n',
          binary='renamed-emulator', busy=True, pids=(4244,))
    check('Linux truncated configured name resolves to running emulator',
          ' 4244 renamed-emulato\n', binary='renamed-emulator', platform='Linux',
          proc_exe='/opt/custom/renamed-emulator', busy=True, pids=(4244,))
    check('Linux matching name prefix belongs to another executable',
          ' 4244 renamed-emulato\n', binary='renamed-emulator', platform='Linux',
          proc_exe='/opt/custom/renamed-emulator-helper')
    check('Linux truncated alias resolves to DOSBox-X',
          ' 4244 renamed-emulato\n', binary='renamed-emulator', platform='Linux',
          proc_exe='/opt/dosbox-x', busy=True, pids=(4244,))
    check('Linux truncated alias resolves to the configured executable',
          ' 4244 renamed-emulato\n', binary='renamed-emulator', platform='Linux',
          symlink_target='actual-emulator', busy=True, pids=(4244,))
    check('Linux unreadable truncated candidate refuses launch',
          ' 4244 renamed-emulato\n', binary='renamed-emulator', platform='Linux',
          readlink_error=True)
    check('idle launch', ' 1 /sbin/launchd\n')
    check('similar names and scripts are not emulator processes',
          ' 11 /usr/bin/python3\n 12 /bin/bash\n 13 dosbox-x-mcp-server\n'
          ' 14 /tmp/dosbox-x-helper\n 15 /tmp/dosbox-run.sh\n')
    check('failed process query refuses launch', '', ps_error=True)
    print('DOSBox launch: busy refusal before probes and writes, idle launch, exact names, Linux truncation, query failure')


if __name__ == '__main__':
    main()
